use std::usize;

use crate::{
    parameters::Params,
    runner::{ComputationKind, DefaultRunner, ParallelRunner},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_iterable::Collection;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

pub struct MapFilterCollect<I, O, Map, Filter, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    map: Map,
    filter: Filter,
    pinned_vec: P,
}

impl<I, O, Map, Filter, P> MapFilterCollect<I, O, Map, Filter, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential_fill_bag(mut self) -> P {
        for x in self.iter.into_seq_iter().map(self.map).filter(self.filter) {
            self.pinned_vec.push(x);
        }
        self.pinned_vec
    }

    fn parallel_compute_in_place<R: ParallelRunner>(self) -> (usize, P) {
        let initial_len = self.iter.try_get_len();
        let offset = self.pinned_vec.len();

        // idx & values has len offset+m where m is the number of added elements
        let idx = ConcurrentOrderedBag::new();
        let values: ConcurrentBag<O, P> = self.pinned_vec.into();
        // pos has len offset+n where n is the length of the input, filtered out values are usize::MAX
        let pos = ConcurrentOrderedBag::new();

        let transform = |(input_idx, value)| {
            let value = (self.map)(value);
            match (self.filter)(&value) {
                true => {
                    let output_idx = values.push(value) - offset;
                    unsafe { pos.set_value(input_idx, output_idx) }; // input_idx in 0..n
                    unsafe { idx.set_value(output_idx, input_idx) }; // output_idx in 0..m
                }
                false => unsafe { pos.set_value(input_idx, usize::MAX) },
            }
        };

        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let num_spawned = runner.run_with_idx(&self.iter, &transform);

        let mut vals = values.into_inner();
        let mut idx = unsafe { idx.into_inner().unwrap_only_if_counts_match() };
        let mut pos = unsafe { pos.into_inner().unwrap_only_if_counts_match() };

        // SAFETY: to avoid reading position values pos_i by index operator such as pos[i]
        // note that we read positions by value and this piece is single threaded
        let pos_write = unsafe { &mut *((&mut pos) as *mut SplitVec<usize>) };

        let mut m = 0;
        for (i, pos_i) in pos.iter().cloned().enumerate() {
            match pos_i {
                usize::MAX => {}       // filtered out
                x if x == m => m += 1, // in place
                _ => {
                    let idx_m = idx[m];
                    debug_assert!(idx_m >= i);
                    debug_assert!(pos_i >= m);

                    vals.swap(offset + m, offset + pos_i);
                    idx.swap(m, pos_i);
                    pos_write[idx_m] = pos_i; // shorthand for: swap(idx_m, i)

                    m += 1;
                }
            }
        }

        debug_assert_eq!(offset + m, vals.len());

        (num_spawned, vals)
    }
}

#[test]
fn abc() {
    use orx_concurrent_iter::*;
    use std::*;

    let offset = 33;
    let n = 159;
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);
    let filter = |x: &String| x.len() > 3;

    let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    let mut expected = Vec::new();

    for i in 0..offset {
        output.push(format!("x{}", i));
        expected.push(format!("x{}", i));
    }

    expected.extend(input.clone().into_iter().map(&map).filter(&filter));

    let mfc = MapFilterCollect {
        iter: input.into_con_iter(),
        params: Default::default(),
        pinned_vec: output,
        filter,
        map,
    };

    let (_, x) = mfc.parallel_compute_in_place::<DefaultRunner>();
    dbg!(&x);

    assert_eq!(expected, x.to_vec());
    assert_eq!(n, 11);
}
