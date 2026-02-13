use crate::algorithms::data_structures::Slice;
use crate::algorithms::merge_sorted_slices::params::ExpSeqMergeSortedSlicesParams;

pub fn seq_merge<'a, T: 'a, F>(
    is_leq: F,
    mut left: &'a Slice<'a, T>,
    mut right: &'a Slice<'a, T>,
    target: &Slice<'a, T>,
    params: ExpSeqMergeSortedSlicesParams,
) where
    F: Fn(&T, &T) -> bool,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != params.put_large_to_left {
        (left, right) = (right, left);
    }
}

fn seq_merge_streak_none<'a, T: 'a, F>(
    is_leq: F,
    left: &'a Slice<'a, T>,
    right: &'a Slice<'a, T>,
    target: &Slice<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut left = left.iter_over_ptr();
    let mut right = right.iter_over_ptr();
    let mut dst = target.iter_as_dst();

    loop {
        unsafe {
            let l = left.current_unchecked();
            let r = right.current_unchecked();

            match is_leq(l, r) {
                true => {
                    dst.write_one_unchecked(left.next_unchecked());
                    if left.is_finished() {
                        dst.write_remaining_from(&right.remaining_into_slice());
                    }
                }
                false => {
                    dst.write_one_unchecked(right.next_unchecked());
                    if right.is_finished() {
                        dst.write_remaining_from(&left.remaining_into_slice());
                    }
                }
            }
        }
    }
}

fn seq_merge_streak_linear<'a, T: 'a, F>(
    is_leq: F,
    left: &'a Slice<'a, T>,
    right: &'a Slice<'a, T>,
    target: &Slice<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut left = left.iter_over_ptr();
    let mut right = right.iter_over_ptr();
    let mut dst = target.iter_as_dst();

    loop {
        unsafe {
            let l = left.current_unchecked();
            let r = right.current_unchecked();

            match is_leq(l, r) {
                true => {
                    let src_begin = left.next_unchecked();
                    let mut src_end_inclusive = src_begin;
                    while let Some(next) = left.next_if_leq(&is_leq, r) {
                        src_end_inclusive = next;
                    }
                    dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if left.is_finished() {
                        dst.write_remaining_from(&right.remaining_into_slice());
                    }
                }
                false => {
                    let src_begin = right.next_unchecked();
                    let mut src_end_inclusive = src_begin;
                    while let Some(next) = right.next_if_leq(&is_leq, l) {
                        src_end_inclusive = next;
                    }
                    dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if right.is_finished() {
                        dst.write_remaining_from(&left.remaining_into_slice());
                    }
                }
            }
        }
    }
}
