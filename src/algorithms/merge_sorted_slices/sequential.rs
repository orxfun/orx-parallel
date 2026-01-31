use crate::algorithms::data_structures::slice_iter::SliceIterDst;
use crate::algorithms::data_structures::{Slice, SliceMut};
use crate::algorithms::merge_sorted_slices::alg::{MergeSortedSlicesParams, StreakSearch};

pub fn merge_sorted_slices<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
    params: MergeSortedSlicesParams,
) where
    F: Fn(&T, &T) -> bool,
{
    debug_assert_eq!(left.len() + right.len(), target.len());

    let mut dst = target.iter_as_dst();

    match (left.len(), right.len()) {
        (0, _) => unsafe { dst.write_remaining_from(right) },
        (_, 0) => unsafe { dst.write_remaining_from(left) },
        _ => match params.streak_search {
            StreakSearch::None => merge_sorted_slices_no_streak(is_leq, left, right, dst),
            StreakSearch::Linear => {
                merge_sorted_slices_with_linear_streak(is_leq, left, right, dst)
            }
            StreakSearch::Binary => todo!(),
        },
    }
}

fn merge_sorted_slices_no_streak<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    mut dst: SliceIterDst<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut left = left.iter_over_ptr();
    let mut right = right.iter_over_ptr();

    loop {
        let (a, b) = unsafe {
            let l = left.current_unchecked();
            let r = right.current_unchecked();

            match is_leq(l, r) {
                true => (&mut left, &mut right),
                false => (&mut right, &mut left),
            }
        };

        unsafe { dst.write_one_unchecked(a.next_unchecked()) };

        if a.is_finished() {
            unsafe { dst.write_remaining_from(&b.remaining_into_slice()) };
            break;
        }
    }
}

fn merge_sorted_slices_with_linear_streak<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    mut dst: SliceIterDst<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut left = left.iter_over_ptr();
    let mut right = right.iter_over_ptr();

    loop {
        let (a, b) = unsafe {
            let l = left.current_unchecked();
            let r = right.current_unchecked();

            match is_leq(l, r) {
                true => (&mut left, &mut right),
                false => (&mut right, &mut left),
            }
        };

        let pivot = unsafe { b.current_unchecked() };
        let src_begin = unsafe { a.next_unchecked() };
        let mut src_end_inclusive = src_begin;

        while let Some(next) = a.next_if_leq(&is_leq, pivot) {
            src_end_inclusive = next;
        }

        unsafe { dst.write_many_unchecked(src_begin, src_end_inclusive) };

        if a.is_finished() {
            unsafe { dst.write_remaining_from(&b.remaining_into_slice()) };
            break;
        }
    }
}
