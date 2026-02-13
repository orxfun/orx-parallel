use crate::algorithms::data_structures::Slice;
use crate::algorithms::exp_merge_sorted_slices::StreakSearch;
use crate::algorithms::exp_merge_sorted_slices::params::ParamsSeqMergeSortedSlices;
use core::cmp::Ordering;

pub fn seq_merge<'a, T: 'a, F>(
    is_leq: F,
    mut left: &'a Slice<'a, T>,
    mut right: &'a Slice<'a, T>,
    target: &Slice<'a, T>,
    params: ParamsSeqMergeSortedSlices,
) where
    F: Fn(&T, &T) -> bool,
{
    let is_large_on_left = left.len() >= right.len();
    if is_large_on_left != params.put_large_to_left {
        (left, right) = (right, left);
    }

    match params.streak_search {
        StreakSearch::None => seq_merge_streak_none(is_leq, left, right, target),
        StreakSearch::Linear => seq_merge_streak_linear(is_leq, left, right, target),
        StreakSearch::Binary => seq_merge_streak_binary(is_leq, left, right, target),
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
    let mut it_left = left.iter_over_ptr();
    let mut it_right = right.iter_over_ptr();
    let mut it_dst = target.iter_as_dst();

    loop {
        unsafe {
            let l = it_left.current_unchecked();
            let r = it_right.current_unchecked();

            match is_leq(l, r) {
                true => {
                    it_dst.write_one_unchecked(it_left.next_unchecked());
                    if it_left.is_finished() {
                        it_dst.write_remaining_from(&it_right.remaining_into_slice());
                    }
                }
                false => {
                    it_dst.write_one_unchecked(it_right.next_unchecked());
                    if it_right.is_finished() {
                        it_dst.write_remaining_from(&it_left.remaining_into_slice());
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
    let mut it_left = left.iter_over_ptr();
    let mut it_right = right.iter_over_ptr();
    let mut it_dst = target.iter_as_dst();

    loop {
        unsafe {
            let l = it_left.current_unchecked();
            let r = it_right.current_unchecked();

            match is_leq(l, r) {
                true => {
                    let src_begin = it_left.next_unchecked();
                    let mut src_end_inclusive = src_begin;
                    while let Some(next) = it_left.next_if_leq(&is_leq, r) {
                        src_end_inclusive = next;
                    }

                    it_dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if it_left.is_finished() {
                        it_dst.write_remaining_from(&it_right.remaining_into_slice());
                    }
                }
                false => {
                    let src_begin = it_right.next_unchecked();
                    let mut src_end_inclusive = src_begin;
                    while let Some(next) = it_right.next_if_leq(&is_leq, l) {
                        src_end_inclusive = next;
                    }

                    it_dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if it_right.is_finished() {
                        it_dst.write_remaining_from(&it_left.remaining_into_slice());
                    }
                }
            }
        }
    }
}

fn seq_merge_streak_binary<'a, T: 'a, F>(
    is_leq: F,
    left: &'a Slice<'a, T>,
    right: &'a Slice<'a, T>,
    target: &Slice<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut it_left = left.iter_over_ptr();
    let mut it_right = right.iter_over_ptr();
    let mut it_dst = target.iter_as_dst();

    loop {
        unsafe {
            let l = it_left.current_unchecked();
            let r = it_right.current_unchecked();

            match is_leq(l, r) {
                true => {
                    let ptr = it_left.peek_unchecked();
                    let remaining = left.subslice_from(ptr).as_slice();
                    let bin_search = remaining.binary_search_by(|x| match is_leq(x, r) {
                        true => Ordering::Less,
                        false => Ordering::Greater,
                    });
                    let idx = match bin_search {
                        Ok(idx) => idx,
                        Err(idx) => idx,
                    };
                    let src_begin = it_left.peek_unchecked();
                    let src_end_inclusive = src_begin.add(idx - 1);
                    it_left.jump_to(src_end_inclusive);

                    it_dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if it_left.is_finished() {
                        it_dst.write_remaining_from(&it_right.remaining_into_slice());
                    }
                }
                false => {
                    let ptr = it_right.peek_unchecked();
                    let remaining = right.subslice_from(ptr).as_slice();
                    let bin_search = remaining.binary_search_by(|x| match is_leq(x, l) {
                        true => Ordering::Less,
                        false => Ordering::Greater,
                    });
                    let idx = match bin_search {
                        Ok(idx) => idx,
                        Err(idx) => idx,
                    };
                    let src_begin = it_right.peek_unchecked();
                    let src_end_inclusive = src_begin.add(idx - 1);
                    it_right.jump_to(src_end_inclusive);

                    it_dst.write_many_unchecked(src_begin, src_end_inclusive);

                    if it_right.is_finished() {
                        it_dst.write_remaining_from(&it_left.remaining_into_slice());
                    }
                }
            }
        }
    }
}
