use crate::{
    parameters::Params,
    runner::{ComputationKind, DefaultRunner, ParallelRunner},
    CollectOrdering,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_iterable::Collection;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
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
    pub fn compute<R: ParallelRunner>(self) -> (usize, P) {
        match (self.params.is_sequential(), self.params.collect_ordering) {
            (true, _) => (0, self.sequential_fill_bag()),
            (false, CollectOrdering::Arbitrary) => todo!(),
            (false, CollectOrdering::SortInPlace) => self.parallel_compute_in_place::<R>(),
            (false, CollectOrdering::SortWithHeap) => self.parallel_compute_heap_sort::<R>(),
        }
    }

    fn sequential_fill_bag(mut self) -> P {
        for x in self.iter.into_seq_iter().map(self.map).filter(self.filter) {
            self.pinned_vec.push(x);
        }
        self.pinned_vec
    }

    fn parallel_compute_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let initial_len = self.iter.try_get_len();

        // values has length of offset+m where m is the number of added elements
        let values: ConcurrentBag<O, P> = self.pinned_vec.into();

        let transform = |value| {
            let value = (self.map)(value);
            if (self.filter)(&value) {
                values.push(value);
            }
        };

        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let num_spawned = runner.run(&self.iter, &transform);

        let values = values.into_inner();
        (num_spawned, values)
    }

    fn parallel_compute_in_place<R: ParallelRunner>(self) -> (usize, P) {
        let initial_len = self.iter.try_get_len();
        let offset = self.pinned_vec.len();

        // idx & values has length of offset+m where m is the number of added elements
        let idx = ConcurrentOrderedBag::new();
        let values: ConcurrentBag<O, P> = self.pinned_vec.into();
        // pos has length of offset+n where n is the length of the input, filtered out values are usize::MAX
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

    fn parallel_compute_heap_sort<R: ParallelRunner>(mut self) -> (usize, P) {
        let initial_len = self.iter.try_get_len();
        let runner = R::new(ComputationKind::Collect, self.params, initial_len);
        let (num_spawned, mut vectors) =
            runner.collect_into_vec_with_idx(&self.iter, &self.map, &self.filter);

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
            self.pinned_vec.push(unsafe { ptr.add(idx).read().1 });
        }

        for vec in vectors.iter_mut() {
            // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
            // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
            unsafe { vec.set_len(0) };
        }

        (num_spawned, self.pinned_vec)
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

    let (_, x) = mfc.parallel_compute_heap_sort::<DefaultRunner>();

    // let (_, x) = mfc.parallel_compute_in_place::<DefaultRunner>();
    dbg!(&x);

    assert_eq!(expected, x.to_vec());
}

#[test]
fn def() {
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

    let (_, mut x) = mfc.parallel_compute_in_arbitrary::<DefaultRunner>();

    // let (_, x) = mfc.parallel_compute_in_place::<DefaultRunner>();
    dbg!(&x);

    expected.sort();
    x.sort();

    assert_eq!(expected, x.to_vec());
}
