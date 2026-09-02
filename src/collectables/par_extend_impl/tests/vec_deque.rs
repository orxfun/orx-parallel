use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::utils::ColAndPos;
use alloc::collections::VecDeque;
use alloc::{vec, vec::Vec};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut deque: VecDeque<i32> = VecDeque::new();
    let results: Vec<ColAndPos<Vec<i32>>> = Vec::new();

    deque.extend_from_ordered_thread_results(results);
    assert!(deque.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut deque: VecDeque<i32> = VecDeque::from([1, 2, 3]);
    let t0 = ColAndPos::<Vec<i32>>::default();
    let t1 = ColAndPos::<Vec<i32>>::default();

    deque.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([1, 2, 3]));
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut deque = VecDeque::new();
    let mut t0 = ColAndPos::default();

    VecDeque::add_ordered_thread_values(&mut t0, 0, vec![10, 20, 30]);

    deque.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(deque, VecDeque::from([10, 20, 30]));
}

#[test]
fn extend_from_ordered_thread_results_single_thread_multiple_chunks() {
    let mut deque = VecDeque::new();
    let mut t0 = ColAndPos::default();

    VecDeque::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    VecDeque::add_ordered_thread_value(&mut t0, 1, 3);
    VecDeque::add_ordered_thread_values(&mut t0, 2, vec![4, 5, 6]);

    deque.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(deque, VecDeque::from([1, 2, 3, 4, 5, 6]));
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads_in_order() {
    let mut deque = VecDeque::new();
    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();

    VecDeque::add_ordered_thread_values(&mut t0, 0, vec![1, 2]);
    VecDeque::add_ordered_thread_values(&mut t0, 2, vec![5, 6]);

    VecDeque::add_ordered_thread_values(&mut t1, 1, vec![3, 4]);
    VecDeque::add_ordered_thread_values(&mut t1, 3, vec![7, 8]);

    deque.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([1, 2, 3, 4, 5, 6, 7, 8]));
}

#[test]
fn extend_from_ordered_thread_results_interleaved_threads() {
    let mut deque = VecDeque::new();
    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();
    let mut t2 = ColAndPos::default();

    // t0 has chunks 3 and 5
    VecDeque::add_ordered_thread_values(&mut t0, 3, vec![7, 8]);
    VecDeque::add_ordered_thread_value(&mut t0, 5, 11);

    // t1 has chunks 0 and 2
    VecDeque::add_ordered_thread_values(&mut t1, 0, vec![1, 2, 3]);
    VecDeque::add_ordered_thread_value(&mut t1, 2, 6);

    // t2 has chunks 1 and 4
    VecDeque::add_ordered_thread_values(&mut t2, 1, vec![4, 5]);
    VecDeque::add_ordered_thread_values(&mut t2, 4, vec![9, 10]);

    deque.extend_from_ordered_thread_results(vec![t0, t1, t2]);
    assert_eq!(deque, VecDeque::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]));
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_deque() {
    let mut deque = VecDeque::from([100, 200]);
    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();

    VecDeque::add_ordered_thread_value(&mut t0, 0, 1);
    VecDeque::add_ordered_thread_value(&mut t1, 1, 2);

    deque.extend_from_ordered_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([100, 200, 1, 2]));
}

#[test]
fn extend_from_ordered_thread_results_empty_iterators_ignored() {
    let mut deque = VecDeque::new();
    let mut t0 = ColAndPos::default();

    // Empty iterator added should not create a chunk
    VecDeque::add_ordered_thread_values(&mut t0, 0, Vec::<i32>::new());
    VecDeque::add_ordered_thread_values(&mut t0, 1, vec![10, 20]);

    deque.extend_from_ordered_thread_results(vec![t0]);
    assert_eq!(deque, VecDeque::from([10, 20]));
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

    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();

    VecDeque::add_ordered_thread_value(&mut t0, 0, DropCounter(drop_count.clone()));
    VecDeque::add_ordered_thread_value(&mut t1, 1, DropCounter(drop_count.clone()));
    VecDeque::add_ordered_thread_value(&mut t0, 2, DropCounter(drop_count.clone()));

    assert_eq!(drop_count.load(Ordering::Relaxed), 0);

    {
        let mut deque = VecDeque::new();
        deque.extend_from_ordered_thread_results(vec![t0, t1]);
        assert_eq!(deque.len(), 3);
        assert_eq!(drop_count.load(Ordering::Relaxed), 0);
    } // deque drops here

    assert_eq!(drop_count.load(Ordering::Relaxed), 3);
}

#[test]
fn extend_from_ordered_thread_results_many_threads_and_chunks() {
    let mut deque = VecDeque::new();
    let num_threads = 8;
    let chunks_per_thread = 10;

    let mut thread_results: Vec<ColAndPos<Vec<(usize, usize)>>> =
        (0..num_threads).map(|_| ColAndPos::default()).collect();

    for chunk_idx in 0..chunks_per_thread {
        for (t_idx, thread_res) in thread_results.iter_mut().enumerate() {
            let global_idx = chunk_idx * num_threads + t_idx;
            let items = vec![(global_idx, 0), (global_idx, 1)];
            VecDeque::add_ordered_thread_values(thread_res, global_idx, items);
        }
    }

    deque.extend_from_ordered_thread_results(thread_results);

    let expected_len = num_threads * chunks_per_thread * 2;
    assert_eq!(deque.len(), expected_len);

    for i in 0..(num_threads * chunks_per_thread) {
        assert_eq!(deque[i * 2], (i, 0));
        assert_eq!(deque[i * 2 + 1], (i, 1));
    }
}

// extend_from_thread_results tests

#[test]
fn extend_from_thread_results_empty() {
    let mut deque: VecDeque<i32> = VecDeque::new();
    let results: Vec<VecDeque<i32>> = Vec::new();

    deque.extend_from_thread_results(results);
    assert!(deque.is_empty());
}

#[test]
fn extend_from_thread_results_empty_threads() {
    let mut deque: VecDeque<i32> = VecDeque::from([1, 2, 3]);
    let t0 = VecDeque::<i32>::new();
    let t1 = VecDeque::<i32>::new();

    deque.extend_from_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([1, 2, 3]));
}

#[test]
fn extend_from_thread_results_single_thread() {
    let mut deque = VecDeque::new();
    let mut t0 = VecDeque::new();

    VecDeque::add_thread_value(&mut t0, 10);
    VecDeque::add_thread_values(&mut t0, vec![20, 30]);

    deque.extend_from_thread_results(vec![t0]);
    assert_eq!(deque, VecDeque::from([10, 20, 30]));
}

#[test]
fn extend_from_thread_results_multiple_threads() {
    let mut deque = VecDeque::new();
    let mut t0 = VecDeque::new();
    let mut t1 = VecDeque::new();

    VecDeque::add_thread_value(&mut t0, 1);
    VecDeque::add_thread_values(&mut t0, vec![2, 3]);

    VecDeque::add_thread_value(&mut t1, 4);
    VecDeque::add_thread_values(&mut t1, vec![5, 6]);

    deque.extend_from_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([1, 2, 3, 4, 5, 6]));
}

#[test]
fn extend_from_thread_results_append_to_non_empty_deque() {
    let mut deque = VecDeque::from([100, 200]);
    let mut t0 = VecDeque::new();
    let mut t1 = VecDeque::new();

    VecDeque::add_thread_value(&mut t0, 1);
    VecDeque::add_thread_value(&mut t1, 2);

    deque.extend_from_thread_results(vec![t0, t1]);
    assert_eq!(deque, VecDeque::from([100, 200, 1, 2]));
}

#[test]
fn extend_from_thread_results_non_copy_drop() {
    struct DropCounter(Arc<AtomicUsize>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    let drop_count = Arc::new(AtomicUsize::new(0));

    let mut t0 = VecDeque::new();
    let mut t1 = VecDeque::new();

    VecDeque::add_thread_value(&mut t0, DropCounter(drop_count.clone()));
    VecDeque::add_thread_value(&mut t1, DropCounter(drop_count.clone()));
    VecDeque::add_thread_value(&mut t0, DropCounter(drop_count.clone()));

    assert_eq!(drop_count.load(Ordering::Relaxed), 0);

    {
        let mut deque = VecDeque::new();
        deque.extend_from_thread_results(vec![t0, t1]);
        assert_eq!(deque.len(), 3);
        assert_eq!(drop_count.load(Ordering::Relaxed), 0);
    }

    assert_eq!(drop_count.load(Ordering::Relaxed), 3);
}
