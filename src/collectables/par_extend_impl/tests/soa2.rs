use crate::collectables::par_extend_core::ParExtendCore;
use crate::collectables::par_extend_impl::utils::ColAndPos;
use crate::{IntoParIter, IterationOrder, Par, ParExtend, Soa2};
use alloc::{string::String, vec, vec::Vec};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn assert_pairs_eq<A, B>(soa: Soa2<A, B>, expected: Vec<(A, B)>)
where
    A: PartialEq + core::fmt::Debug,
    B: PartialEq + core::fmt::Debug,
{
    let (v1, v2) = soa.into_inner();
    let actual: Vec<(A, B)> = v1.into_iter().zip(v2).collect();
    assert_eq!(actual, expected);
}

fn assert_pairs_sorted_eq<A, B>(soa: Soa2<A, B>, mut expected: Vec<(A, B)>)
where
    A: Ord + core::fmt::Debug,
    B: Ord + core::fmt::Debug,
{
    let (v1, v2) = soa.into_inner();
    let mut actual: Vec<(A, B)> = v1.into_iter().zip(v2).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn extend_from_ordered_thread_results_empty() {
    let mut soa: Soa2<i32, usize> = Soa2::new();
    let results: Vec<ColAndPos<Soa2<i32, usize>>> = Vec::new();

    soa.extend_merge_ordered_infallibles(results);
    let (v1, v2) = soa.into_inner();
    assert!(v1.is_empty());
    assert!(v2.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_empty_threads() {
    let mut soa: Soa2<i32, usize> = Soa2::new();
    let t0 = ColAndPos::<Soa2<i32, usize>>::default();
    let t1 = ColAndPos::<Soa2<i32, usize>>::default();

    soa.extend_merge_ordered_infallibles(vec![t0, t1]);
    let (v1, v2) = soa.into_inner();
    assert!(v1.is_empty());
    assert!(v2.is_empty());
}

#[test]
fn extend_from_ordered_thread_results_single_thread_single_chunk() {
    let mut soa = Soa2::new();
    let mut t0 = ColAndPos::default();

    Soa2::add_ordered_thread_values(&mut t0, 0, vec![(10, 100_usize), (20, 200), (30, 300)]);

    soa.extend_merge_ordered_infallibles(vec![t0]);
    assert_pairs_eq(soa, vec![(10, 100), (20, 200), (30, 300)]);
}

#[test]
fn extend_from_ordered_thread_results_multiple_threads_in_order() {
    let mut soa = Soa2::new();
    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();

    Soa2::add_ordered_thread_values(&mut t0, 0, vec![(1, 10_usize), (2, 20)]);
    Soa2::add_ordered_thread_values(&mut t0, 2, vec![(5, 50), (6, 60)]);

    Soa2::add_ordered_thread_values(&mut t1, 1, vec![(3, 30), (4, 40)]);
    Soa2::add_ordered_thread_values(&mut t1, 3, vec![(7, 70), (8, 80)]);

    soa.extend_merge_ordered_infallibles(vec![t0, t1]);
    assert_pairs_eq(
        soa,
        vec![
            (1, 10),
            (2, 20),
            (3, 30),
            (4, 40),
            (5, 50),
            (6, 60),
            (7, 70),
            (8, 80),
        ],
    );
}

#[test]
fn extend_from_ordered_thread_results_interleaved_threads() {
    let mut soa = Soa2::new();
    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();
    let mut t2 = ColAndPos::default();

    Soa2::add_ordered_thread_values(&mut t0, 3, vec![(7, 70), (8, 80)]);
    Soa2::add_ordered_thread_value(&mut t0, 5, (11, 110));

    Soa2::add_ordered_thread_values(&mut t1, 0, vec![(1, 10), (2, 20), (3, 30)]);
    Soa2::add_ordered_thread_value(&mut t1, 2, (6, 60));

    Soa2::add_ordered_thread_values(&mut t2, 1, vec![(4, 40), (5, 50)]);
    Soa2::add_ordered_thread_values(&mut t2, 4, vec![(9, 90), (10, 100)]);

    soa.extend_merge_ordered_infallibles(vec![t0, t1, t2]);
    assert_pairs_eq(
        soa,
        vec![
            (1, 10),
            (2, 20),
            (3, 30),
            (4, 40),
            (5, 50),
            (6, 60),
            (7, 70),
            (8, 80),
            (9, 90),
            (10, 100),
            (11, 110),
        ],
    );
}

#[test]
fn extend_from_ordered_thread_results_append_to_non_empty_soa() {
    let mut soa = Soa2::new();
    soa.extend(vec![(100, 1000_usize), (200, 2000)]);

    let mut t0 = ColAndPos::default();
    let mut t1 = ColAndPos::default();

    Soa2::add_ordered_thread_value(&mut t0, 0, (1, 10_usize));
    Soa2::add_ordered_thread_value(&mut t1, 1, (2, 20));

    soa.extend_merge_ordered_infallibles(vec![t0, t1]);
    assert_pairs_eq(soa, vec![(100, 1000), (200, 2000), (1, 10), (2, 20)]);
}

#[test]
fn extend_from_ordered_thread_results_empty_iterators_ignored() {
    let mut soa = Soa2::new();
    let mut t0 = ColAndPos::default();

    Soa2::add_ordered_thread_values(&mut t0, 0, Vec::<(i32, usize)>::new());
    Soa2::add_ordered_thread_values(&mut t0, 1, vec![(10, 100), (20, 200)]);

    soa.extend_merge_ordered_infallibles(vec![t0]);
    assert_pairs_eq(soa, vec![(10, 100), (20, 200)]);
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

    Soa2::add_ordered_thread_value(
        &mut t0,
        0,
        (DropCounter(drop_count.clone()), String::from("alpha")),
    );
    Soa2::add_ordered_thread_value(
        &mut t1,
        1,
        (DropCounter(drop_count.clone()), String::from("beta")),
    );
    Soa2::add_ordered_thread_value(
        &mut t0,
        2,
        (DropCounter(drop_count.clone()), String::from("gamma")),
    );

    assert_eq!(drop_count.load(Ordering::Relaxed), 0);

    {
        let mut soa = Soa2::new();
        soa.extend_merge_ordered_infallibles(vec![t0, t1]);
        let (v1, v2) = soa.into_inner();
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 3);
        assert_eq!(drop_count.load(Ordering::Relaxed), 0);
    }

    assert_eq!(drop_count.load(Ordering::Relaxed), 3);
}

#[test]
fn extend_from_thread_results_empty() {
    let mut soa: Soa2<i32, usize> = Soa2::new();
    let results: Vec<Soa2<i32, usize>> = Vec::new();

    soa.extend_merge_infallibles(results);
    let (v1, v2) = soa.into_inner();
    assert!(v1.is_empty());
    assert!(v2.is_empty());
}

#[test]
fn extend_from_thread_results_multiple_threads() {
    let mut soa = Soa2::new();
    let mut t0 = Soa2::new();
    let mut t1 = Soa2::new();

    Soa2::add_thread_value(&mut t0, (1, 10_usize));
    Soa2::add_thread_values(&mut t0, vec![(2, 20), (3, 30)]);

    Soa2::add_thread_value(&mut t1, (4, 40));
    Soa2::add_thread_values(&mut t1, vec![(5, 50), (6, 60)]);

    soa.extend_merge_infallibles(vec![t0, t1]);
    assert_pairs_eq(
        soa,
        vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
    );
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

    let mut t0 = Soa2::new();
    let mut t1 = Soa2::new();

    Soa2::add_thread_value(
        &mut t0,
        (DropCounter(drop_count.clone()), String::from("alpha")),
    );
    Soa2::add_thread_value(
        &mut t1,
        (DropCounter(drop_count.clone()), String::from("beta")),
    );
    Soa2::add_thread_value(
        &mut t0,
        (DropCounter(drop_count.clone()), String::from("gamma")),
    );

    assert_eq!(drop_count.load(Ordering::Relaxed), 0);

    {
        let mut soa = Soa2::new();
        soa.extend_merge_infallibles(vec![t0, t1]);
        let (v1, v2) = soa.into_inner();
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 3);
        assert_eq!(drop_count.load(Ordering::Relaxed), 0);
    }

    assert_eq!(drop_count.load(Ordering::Relaxed), 3);
}

#[test]
fn par_extend_ordered() {
    let input: Vec<(i32, usize)> = (0..100).map(|i| (i, (i * 10) as usize)).collect();
    let mut soa = Soa2::new();

    soa.par_extend(
        input
            .clone()
            .into_par()
            .iteration_order(IterationOrder::Ordered)
            .map(|(a, b)| (a * 2, b * 2)),
    );

    assert_pairs_eq(
        soa,
        input.into_iter().map(|(a, b)| (a * 2, b * 2)).collect(),
    );
}

#[test]
fn par_extend_arbitrary() {
    let input: Vec<(i32, usize)> = (0..100).map(|i| (i, (i * 10) as usize)).collect();
    let mut soa = Soa2::new();

    soa.par_extend(
        input
            .clone()
            .into_par()
            .iteration_order(IterationOrder::Arbitrary)
            .map(|(a, b)| (a * 2, b * 2)),
    );

    assert_pairs_sorted_eq(
        soa,
        input.into_iter().map(|(a, b)| (a * 2, b * 2)).collect(),
    );
}
