use super::mfm::Mfm;
use crate::{
    runner::{ComputationKind, ParallelRunner},
    CollectOrdering,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_iterable::Collection;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::SplitVec;

pub struct MfmCollect<I, T, O, Map1, Filter, Map2, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T + Send + Sync,
    Filter: Fn(&T) -> bool + Send + Sync,
    Map2: Fn(T) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
    O: Send + Sync,
{
    mfm: Mfm<I, T, O, Map1, Filter, Map2>,
    pinned_vec: P,
}

impl<I, T, O, Map1, Filter, Map2, P> MfmCollect<I, T, O, Map1, Filter, Map2, P>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T + Send + Sync,
    Filter: Fn(&T) -> bool + Send + Sync,
    Map2: Fn(T) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
    O: Send + Sync,
{
    pub fn compute<R: ParallelRunner>(self) -> (usize, P) {
        let params = &self.mfm.params;
        match (params.is_sequential(), params.collect_ordering) {
            (true, _) => (0, self.sequential_fill_bag()),
            (false, CollectOrdering::Arbitrary) => todo!(),
            (false, CollectOrdering::SortInPlace) => self.parallel_compute_in_place::<R>(),
            (false, CollectOrdering::SortWithHeap) => self.parallel_compute_heap_sort::<R>(),
        }
    }

    fn sequential_fill_bag(self) -> P {
        let (mfm, mut pinned_vec) = (self.mfm, self.pinned_vec);

        for x in mfm
            .iter
            .into_seq_iter()
            .map(mfm.map1)
            .filter(mfm.filter)
            .map(mfm.map2)
        {
            pinned_vec.push(x);
        }
        pinned_vec
    }

    fn parallel_compute_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, pinned_vec) = (self.mfm, self.pinned_vec);

        let initial_len = mfm.iter.try_get_len();

        // values has length of offset+m where m is the number of added elements
        let values: ConcurrentBag<O, P> = pinned_vec.into();

        let transform = |value| {
            let value = (mfm.map1)(value);
            if (mfm.filter)(&value) {
                values.push((mfm.map2)(value));
            }
        };

        let runner = R::new(ComputationKind::Collect, mfm.params, initial_len);
        let num_spawned = runner.run(&mfm.iter, &transform);

        let values = values.into_inner();
        (num_spawned, values)
    }

    fn parallel_compute_in_place<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, pinned_vec) = (self.mfm, self.pinned_vec);

        let initial_len = mfm.iter.try_get_len();
        let offset = pinned_vec.len();

        // idx & values has length of offset+m where m is the number of added elements
        let idx = ConcurrentOrderedBag::new();
        let values: ConcurrentBag<O, P> = pinned_vec.into();
        // pos has length of offset+n where n is the length of the input, filtered out values are usize::MAX
        let pos = ConcurrentOrderedBag::new();

        let transform = |(input_idx, value)| {
            let value = (mfm.map1)(value);
            match (mfm.filter)(&value) {
                true => {
                    let output_idx = values.push((mfm.map2)(value)) - offset;
                    unsafe { pos.set_value(input_idx, output_idx) }; // input_idx in 0..n
                    unsafe { idx.set_value(output_idx, input_idx) }; // output_idx in 0..m
                }
                false => unsafe { pos.set_value(input_idx, usize::MAX) },
            }
        };

        let runner = R::new(ComputationKind::Collect, mfm.params, initial_len);
        let num_spawned = runner.run_with_idx(&mfm.iter, &transform);

        let mut values = values.into_inner();
        let mut idx = unsafe { idx.into_inner().unwrap_only_if_counts_match() };
        let mut pos = unsafe { pos.into_inner().unwrap_only_if_counts_match() };

        // SAFETY: to avoid reading position values pos_i by index operator such as pos[i]
        // note that we read positions by value and this code block is single threaded
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

                    values.swap(offset + m, offset + pos_i);
                    idx.swap(m, pos_i);
                    pos_write[idx_m] = pos_i; // shorthand for: swap(idx_m, i)

                    m += 1;
                }
            }
        }

        debug_assert_eq!(offset + m, values.len());

        (num_spawned, values)
    }

    fn parallel_compute_heap_sort<R: ParallelRunner>(self) -> (usize, P) {
        let (mfm, mut pinned_vec) = (self.mfm, self.pinned_vec);

        let initial_len = mfm.iter.try_get_len();
        let runner = R::new(ComputationKind::Collect, mfm.params, initial_len);

        let (num_spawned, mut vectors) =
            runner.mfm_collect_with_idx(&mfm.iter, &mfm.map1, &mfm.filter, &mfm.map2);

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
}
