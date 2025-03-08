use crate::{
    parameters::Params,
    runner::{ComputationKind, DefaultRunner, ParallelRunner},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
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
    pinned: P,
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
            self.pinned.push(x);
        }
        self.pinned
    }

    fn parallel_compute_in_place<R: ParallelRunner>(self) -> (usize, ConcurrentBag<O, P>) {
        let initial_len = self.iter.try_get_len();

        let mut indices = SplitVec::new();
        for i in 0..self.pinned.len() {
            indices.push(i);
        }
        let indices = ConcurrentOrderedBag::from(indices);

        let values: ConcurrentBag<O, P> = self.pinned.into();

        let transform = |(input_idx, value)| {
            let value = (self.map)(value);
            if (self.filter)(&value) {
                let values_idx = values.push(value);
                unsafe { indices.set_value(input_idx, values_idx) };
            }
        };

        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let num_spawned = runner.run_with_idx(&self.iter, &transform);

        (num_spawned, values)
    }
}

#[test]
fn abc() {
    use orx_concurrent_iter::*;
    use std::*;

    let n = 25;
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let map = |x: String| format!("{}!", x);
    let filter = |x: &String| x.len() > 2;

    let expected: Vec<_> = input
        .clone()
        .into_iter()
        .map(&map)
        .filter(&filter)
        .collect();

    let mfc = MapFilterCollect {
        iter: input.into_con_iter(),
        params: Default::default(),
        pinned: SplitVec::new(),
        filter,
        map,
    };

    // let (_, x) = mfc.parallel_compute_in_place::<DefaultRunner>();
    // let x = x.into_inner();
    // dbg!(&x);

    // assert_eq!(expected, x.to_vec());
    // assert_eq!(n, 11);
}
