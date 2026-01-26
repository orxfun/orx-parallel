use crate::sort::slice::{SortChunks, sort2};
use crate::sort::{slice::sort, tests::utils::create_input_and_sorted};
use crate::{ParThreadPool, StdDefaultPool};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use test_case::{test_case, test_matrix};

#[test_case(0, 0, vec![])]
#[test_case(0, 1, vec![])]
#[test_case(10, 0, vec![])]
#[test_case(15, 5, vec![vec![0,1,2], vec![3,4,5], vec![6,7,8], vec![9,10,11], vec![12,13,14]])]
#[test_case(15, 4, vec![vec![0,1,2,3], vec![4,5,6,7], vec![8,9,10,11], vec![12,13,14]])]
#[test_case(15, 2, vec![vec![0,1,2,3,4,5,6,7], vec![8,9,10,11,12,13,14]])]
#[test_case(15, 1, vec![vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14]])]
#[test_case(15, 6, vec![vec![0,1,2], vec![3,4,5], vec![6,7,8], vec![9,10], vec![11,12], vec![13,14]])]
#[test_case(15, 10, vec![vec![0,1], vec![2,3], vec![4,5], vec![6,7], vec![8,9], vec![10], vec![11], vec![12], vec![13], vec![14]])]
fn slice_chunks(len: usize, num_chunks: usize, chunks: Vec<Vec<usize>>) {
    let mut v: Vec<_> = (0..len).collect();
    assert_eq!(
        crate::sort::slice_chunk::slice_chunks(v.as_mut_ptr(), v.len(), num_chunks)
            .into_iter()
            .map(|c| c.as_mut_slice().iter().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        chunks
    );
}

#[test_matrix(
    [0, 1, 1034],
    [0, 10000],
    [
        StdDefaultPool::default(),
        StdDefaultPool::with_max_num_threads(NonZeroUsize::new(1).unwrap()),
        StdDefaultPool::with_max_num_threads(NonZeroUsize::new(4).unwrap()),
    ],
    [1, 4, 10]
)]
fn slice_sort<P>(len: usize, number_of_swaps: usize, mut pool: P, nt: usize)
where
    P: ParThreadPool,
{
    let nt = NonZeroUsize::new(nt).unwrap();
    let (mut input, sorted) = create_input_and_sorted(len, |i| Box::new(i), number_of_swaps);
    sort(
        &mut pool,
        nt,
        &mut input,
        SortChunks::SeqWithPriorityQueuePtrs,
    );
    assert_eq!(input, sorted);
}

#[test]
fn abc() {
    let len = 1 << 22;
    let number_of_swaps = 4 * len;
    let mut pool = StdDefaultPool::default();
    let nt = 32;

    let nt = NonZeroUsize::new(nt).unwrap();
    let (mut input, sorted) = create_input_and_sorted(len, |i| i, number_of_swaps);
    // println!("before = {input:?}");
    // std::println!("before\n{input:?}");
    // sort(
    //     &mut pool,
    //     nt,
    //     &mut input,
    //     SortChunks::SeqWithPriorityQueuePtrs,
    // );
    sort2(&mut input, 10);
    // println!("after = {input:?}");*

    assert_eq!(input, sorted);
    // std::println!("after\n{input:?}");
    assert_eq!(input.len(), 33);
}
