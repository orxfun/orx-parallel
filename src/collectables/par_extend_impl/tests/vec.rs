use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::vec::VecAndPositions;
use alloc::{vec, vec::Vec};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut vec: Vec<i32> = Vec::new();
    let results: Vec<VecAndPositions<i32>> = Vec::new();

    vec.extend_from_ordered_thread_results(results);
    assert!(vec.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut vec: Vec<i32> = vec![1, 2, 3];
    let t0 = VecAndPositions::<i32>::default();
    let t1 = VecAndPositions::<i32>::default();

    vec.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(vec, vec![1, 2, 3]);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut vec = Vec::new();
    let mut t0 = VecAndPositions::default();

    Vec::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    vec.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(vec, vec![10, 20, 30]);
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut vec = Vec::new();
    let mut t0 = VecAndPositions::default();

    Vec::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    Vec::add_ordered_thread_value(&mut t0, 1, 3);
    Vec::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    vec.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads_in_order() {
    let mut vec = Vec::new();
    let mut t0 = VecAndPositions::default();
    let mut t1 = VecAndPositions::default();

    Vec::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    Vec::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    Vec::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    Vec::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    vec.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn extend_from_ordered_thread_results_interleaved_threads() {
    let mut vec = Vec::new();
    let mut t0 = VecAndPositions::default();
    let mut t1 = VecAndPositions::default();
    let mut t2 = VecAndPositions::default();

    // t0 has chunks 3 and 5
    Vec::add_ordered_thread_values(&mut t0, 3, vec![7, 8]);
    Vec::add_ordered_thread_value(&mut t0, 5, 11);

    // t1 has chunks 0 and 2
    Vec::add_ordered_thread_values(&mut t1, 0, vec![1, 2, 3]);
    Vec::add_ordered_thread_value(&mut t1, 2, 6);

    // t2 has chunks 1 and 4
    Vec::add_ordered_thread_values(&mut t2, 1, vec![4, 5]);
    Vec::add_ordered_thread_values(&mut t2, 4, vec![9, 10]);

    vec.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_vec() {
    let mut vec = vec![100, 200];
    let mut t0 = VecAndPositions::default();
    let mut t1 = VecAndPositions::default();

    Vec::add_ordered_thread_value(&mut t0, 0, 1);
    Vec::add_ordered_thread_value(&mut t1, 1, 2);

    vec.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(vec, vec![100, 200, 1, 2]);
}

#[test]
fn extend_from_ordered_thread_results_empty_iterators_ignored() {
    let mut vec = Vec::new();
    let mut t0 = VecAndPositions::default();

    // Empty iterator added should not create a chunk
    Vec::add_ordered_thread_values(&mut t0, 0, Vec::<i32>::new());
    Vec::add_ordered_thread_values(&mut t0, 1, vec![10, 20]);

    vec.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(vec, vec![10, 20]);
}

#[test]
fn extend_from_ordered_thread_results_non_copy_drop() {
    struct DropCounter(Arc<AtomicUsize>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    let drop_count = Arc::new(AtomicUsize::new(0));

    let mut t0 = VecAndPositions::default();
    let mut t1 = VecAndPositions::default();

    Vec::add_ordered_thread_value(&mut t0, 0, DropCounter(drop_count.clone()));
    Vec::add_ordered_thread_value(&mut t1, 1, DropCounter(drop_count.clone()));
    Vec::add_ordered_thread_value(&mut t0, 2, DropCounter(drop_count.clone()));

    assert_eq!(drop_count.load(Ordering::Relaxed), 0);

    {
        let mut vec = Vec::new();
        vec.extend_from_ordered_thread_results(vec![t0, t1]);
        assert_eq!(vec.len(), 3);
        assert_eq!(drop_count.load(Ordering::Relaxed), 0);
    } // vec drops here

    assert_eq!(drop_count.load(Ordering::Relaxed), 3);
}

#[test]
fn extend_from_ordered_thread_results_many_threads_and_chunks() {
    let mut vec = Vec::new();
    let num_threads = 8;
    let chunks_per_thread = 10;

    let mut thread_results: Vec<VecAndPositions<(usize, usize)>> = (0..num_threads)
        .map(|_| VecAndPositions::default())
        .collect();

    for chunk_idx in 0..chunks_per_thread {
        for (t_idx, thread_res) in thread_results.iter_mut().enumerate() {
            let global_idx = chunk_idx * num_threads + t_idx;
            let items = vec![(global_idx, 0), (global_idx, 1)];
            Vec::add_ordered_thread_values(thread_res, global_idx, items);
        }
    }

    vec.extend_from_ordered_thread_results(thread_results);

    let expected_len = num_threads * chunks_per_thread * 2;
    assert_eq!(vec.len(), expected_len);

    for i in 0..(num_threads * chunks_per_thread) {
        assert_eq!(vec[i * 2], (i, 0));
        assert_eq!(vec[i * 2 + 1], (i, 1));
    }
}
