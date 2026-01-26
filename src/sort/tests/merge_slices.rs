use crate::sort::tests::utils::create_input_and_sorted;
use alloc::boxed::Box;

fn slice_sort(len: usize, number_of_swaps: usize) {
    let (mut input, sorted) = create_input_and_sorted(len, |i| Box::new(i), number_of_swaps);

    assert_eq!(input, sorted);
}
