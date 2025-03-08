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

        let idx =
            ConcurrentOrderedBag::from(SplitVec::with_doubling_growth_and_fragments_capacity(32));
        let pos =
            ConcurrentOrderedBag::from(SplitVec::with_doubling_growth_and_fragments_capacity(32));
        let values: ConcurrentBag<O, P> = self.pinned_vec.into();

        let transform = |(input_idx, value)| {
            let value = (self.map)(value);
            match (self.filter)(&value) {
                true => {
                    let output_idx = values.push(value);
                    unsafe { pos.set_value(input_idx, output_idx) };
                    unsafe { idx.set_value(output_idx, input_idx) };
                }
                false => unsafe { pos.set_value(input_idx, usize::MAX) },
            }
        };

        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let num_spawned = runner.run_with_idx(&self.iter, &transform);

        let mut vals = values.into_inner();
        let mut idx = unsafe { idx.into_inner().unwrap_only_if_counts_match() };
        let mut pos = unsafe { pos.into_inner().unwrap_only_if_counts_match() };

        let mut i = 0;
        for p in 0..pos.len() {
            let pi = pos[p];
            if pi < usize::MAX {
                if i != pi {
                    let ii = idx[i];

                    vals.swap(i, pi);
                    idx.swap(i, pi);
                    pos.swap(i, ii);
                }

                i += 1;
            }
        }

        (num_spawned, vals)
    }
}

#[test]
fn abc() {
    use orx_concurrent_iter::*;
    use std::*;

    let n = 1000;
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);
    let filter = |x: &String| x.len() > 0;

    let expected: Vec<_> = input
        .clone()
        .into_iter()
        .map(&map)
        .filter(&filter)
        .collect();

    let mfc = MapFilterCollect {
        iter: input.into_con_iter(),
        params: Default::default(),
        pinned_vec: SplitVec::with_doubling_growth_and_fragments_capacity(32),
        filter,
        map,
    };

    let (_, x) = mfc.parallel_compute_in_place::<DefaultRunner>();
    dbg!(&x);

    assert_eq!(expected, x.to_vec());
    // assert_eq!(n, 11);
}
