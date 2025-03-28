use std::marker::PhantomData;

use super::mfm::Mfm;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelTask};
use crate::CollectOrdering;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_iterable::Collection;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};

pub struct MfmCollect<I, T, Vt, O, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values<Item = T> + Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    T: Send + Sync,
    O: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
    pinned_vec: P,
}

impl<I, T, Vt, O, Vo, M1, F, M2, P> MfmCollect<I, T, Vt, O, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values<Item = T> + Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    T: Send + Sync,
    O: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    pub fn compute<R: ParallelRunner>(
        mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
        in_input_order: bool,
        pinned_vec: P,
    ) -> (usize, P) {
        let mfm_collect = Self { mfm, pinned_vec };
        let params = mfm_collect.mfm.params();
        match (
            params.is_sequential(),
            in_input_order,
            params.collect_ordering,
        ) {
            (true, _, _) => (0, mfm_collect.sequential()),
            (false, true, _) => mfm_collect.parallel_in_input_order::<R>(),
            (false, false, CollectOrdering::Arbitrary) => mfm_collect.parallel_in_arbitrary::<R>(),
            (false, false, CollectOrdering::SortWithHeap) => {
                mfm_collect.parallel_with_heap_sort::<R>()
            }
        }
    }

    fn sequential(self) -> P {
        let (mfm, mut pinned_vec) = (self.mfm, self.pinned_vec);
        let (_, iter, map1, filter, map2) = mfm.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = map1(i);
            vt.filter_map_collect_sequential(&filter, &map2, &mut pinned_vec);
        }

        pinned_vec
    }

    fn parallel_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, pinned_vec) = (self.mfm, self.pinned_vec);
        let (params, iter, map1, filter, map2) = mfm.destruct();

        // values has length of offset+m where m is the number of added elements
        let bag: ConcurrentBag<O, P> = pinned_vec.into();

        let task = MfmCollectInArbitraryOrder::<'_, I, T, Vt, O, Vo, M1, F, M2, P>::new(
            map1, filter, map2, &bag,
        );

        let runner = R::new(ComputationKind::Collect, params, iter.try_get_len());
        let num_spawned = runner.new_run(&iter, task);

        let values = bag.into_inner();
        (num_spawned, values)
    }

    fn parallel_with_heap_sort<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, mut pinned_vec) = (self.mfm, self.pinned_vec);
        let (params, iter, map1, filter, map2) = mfm.destruct();
        let initial_len = iter.try_get_len();

        let runner = R::new(ComputationKind::Collect, params, initial_len);

        let (num_spawned, mut vectors) = runner.mfm_collect_to_vecs(&iter, &map1, &filter, &map2);

        let mut queue = BinaryHeap::with_capacity(vectors.len());
        let mut indices = vec![0; vectors.len()];

        for (v, vec) in vectors.iter().enumerate() {
            if let Some(x) = vec.get(indices[v]) {
                queue.push(v, x.0);
            }
        }
        let mut curr_v = queue.pop_node();

        while let Some(v) = curr_v {
            let idx = indices[v];
            indices[v] += 1;

            curr_v = match vectors[v].get(indices[v]) {
                Some(x) => Some(queue.push_then_pop(v, x.0).0),
                None => queue.pop_node(),
            };

            let ptr = vectors[v].as_mut_ptr();
            pinned_vec.push(unsafe { ptr.add(idx).read().1 });
        }

        for vec in vectors.iter_mut() {
            // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
            // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
            unsafe { vec.set_len(0) };
        }

        (num_spawned, pinned_vec)
    }

    fn parallel_in_input_order<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, pinned_vec) = (self.mfm, self.pinned_vec);
        let offset = pinned_vec.len();
        let (params, iter, map1, filter, map2) = mfm.destruct();
        let initial_len = iter.try_get_len();

        let o_bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();

        let transform = |(i_idx, i): (usize, I::Item)| {
            let vt = map1(i);
            vt.filter_map_collect_in_input_order(offset + i_idx, &filter, &map2, &o_bag);
        };

        let runner = R::new(ComputationKind::Collect, params, initial_len);
        let num_spawned = runner.run_with_idx(&iter, &transform);

        let values = unsafe { o_bag.into_inner().unwrap_only_if_counts_match() };
        (num_spawned, values)
    }
}

// arbitrary

struct MfmCollectInArbitraryOrder<'a, I, T, Vt, O, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values<Item = T> + Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    T: Send + Sync,
    O: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    map1: M1,
    filter: F,
    map2: M2,
    bag: &'a ConcurrentBag<O, P>,
    phantom: PhantomData<(I, T, Vt, Vo)>,
}

impl<'a, I, T, Vt, O, Vo, M1, F, M2, P>
    MfmCollectInArbitraryOrder<'a, I, T, Vt, O, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values<Item = T> + Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    T: Send + Sync,
    O: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn new(map1: M1, filter: F, map2: M2, bag: &'a ConcurrentBag<O, P>) -> Self {
        Self {
            map1,
            filter,
            map2,
            bag,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, Vt, O, Vo, M1, F, M2, P> ParallelTask
    for MfmCollectInArbitraryOrder<'a, I, T, Vt, O, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values<Item = T> + Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    T: Send + Sync,
    O: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    type Item = I::Item;

    #[inline]
    fn f1(&self, value: Self::Item) {
        let values_vt = (self.map1)(value);
        values_vt.filter_map_collect_arbitrary(&self.filter, &self.map2, &self.bag);
    }

    #[inline(always)]
    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>) {
        for x in values {
            self.f1(x);
        }
    }
}
