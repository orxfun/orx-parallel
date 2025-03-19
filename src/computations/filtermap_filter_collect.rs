use crate::{
    runner::{ComputationKind, ParallelRunner},
    Maybe, Params,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct FiltermapFilterCollect<I, MO, O, FilterMap, Filter, P>
where
    I: ConcurrentIter,
    MO: Maybe<O> + Send + Sync,
    O: Send + Sync,
    FilterMap: Fn(I::Item) -> MO + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Filter,
    pinned_vec: P,
    phantom: PhantomData<O>,
}

impl<I, MO, O, FilterMap, Filter, P> FiltermapFilterCollect<I, MO, O, FilterMap, Filter, P>
where
    I: ConcurrentIter,
    MO: Maybe<O> + Send + Sync,
    O: Send + Sync,
    FilterMap: Fn(I::Item) -> MO + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential_fill_bag(mut self) -> P {
        let iter = self.iter.into_seq_iter();
        for maybe in iter.map(self.filter_map) {
            if maybe.has_value() {
                let value = maybe.value_unchecked();
                if (self.filter)(&value) {
                    self.pinned_vec.push(value);
                }
            }
        }
        self.pinned_vec
    }

    fn parallel_compute_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let initial_len = self.iter.try_get_len();

        // values has length of offset+m where m is the number of added elements
        let values: ConcurrentBag<O, P> = self.pinned_vec.into();

        let transform = |value| {
            let maybe = (self.filter_map)(value);
            if maybe.has_value() {
                let value = maybe.value_unchecked();
                if (self.filter)(&value) {
                    values.push(value);
                }
            }
        };

        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let num_spawned = runner.run(&self.iter, &transform);

        let values = values.into_inner();
        (num_spawned, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::DefaultRunner;
    use orx_concurrent_iter::IntoConcurrentIter;
    use orx_split_vec::{PinnedVec, SplitVec};

    #[test]
    fn xyz_seq() {
        let offset = 33;
        let n = 159;
        let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        let filter_map = |x: String| x.parse().ok();
        let filter = |x: &usize| x > &3;

        let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
        let mut expected = Vec::new();

        for i in 0..offset {
            output.push(i);
            expected.push(i);
        }

        expected.extend(
            input
                .clone()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter),
        );

        let mfc = FiltermapFilterCollect {
            iter: input.into_con_iter(),
            params: Default::default(),
            pinned_vec: output,
            filter,
            filter_map,
            phantom: Default::default(),
        };

        let x = mfc.sequential_fill_bag();

        dbg!(&x);

        assert_eq!(expected, x.to_vec());
    }
    #[test]
    fn xyz_arb() {
        let offset = 33;
        let n = 159;
        let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        let filter_map = |x: String| x.parse().ok();
        let filter = |x: &usize| x > &3;

        let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
        let mut expected = Vec::new();

        for i in 0..offset {
            output.push(i);
            expected.push(i);
        }

        expected.extend(
            input
                .clone()
                .into_iter()
                .filter_map(&filter_map)
                .filter(&filter),
        );

        let mfc = FiltermapFilterCollect {
            iter: input.into_con_iter(),
            params: Default::default(),
            pinned_vec: output,
            filter,
            filter_map,
            phantom: Default::default(),
        };

        let (_, mut x) = mfc.parallel_compute_in_arbitrary::<DefaultRunner>();

        x.sort();
        expected.sort();

        dbg!(&x);

        assert_eq!(expected, x.to_vec());
    }
}
