use crate::{
    parameters::Params,
    runner::{ComputationKind, ParallelRunner},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
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
        let offset = self.pinned.len();
        let initial_len = self.iter.try_get_len();

        let values: ConcurrentBag<O, P> = self.pinned.into();
        let indices = ConcurrentOrderedBag::with_doubling_growth();

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
