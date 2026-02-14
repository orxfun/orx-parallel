use crate::experiment::data_structures::{
    slice::Slice, slice_dst::SliceDst, slice_iter_ptr::SliceIterPtr, slice_src::SliceSrc,
};

#[derive(Clone, Copy)]
pub enum StreakSearch {
    None,
    Linear,
    Binary,
}

#[derive(Clone, Copy)]
pub struct ParamsSeqMergeSortedSlices {
    pub streak_search: StreakSearch,
    pub put_large_to_left: bool,
}

/// # SAFETY
///
/// - (i) length of `target` must equal sum of lengths of `left` and `right`.
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
pub unsafe fn seq_merge<'a, T: 'a, F>(
    is_leq: F,
    mut left: SliceSrc<T>,
    mut right: SliceSrc<T>,
    target: SliceDst<T>,
    params: ParamsSeqMergeSortedSlices,
) where
    F: Fn(&T, &T) -> bool,
{
    debug_assert_eq!(target.len(), left.len() + right.len());

    match (left.len(), right.len()) {
        // // SAFETY: satisfied by (i) and (ii)
        // (0, _) => unsafe { target.copy_from_nonoverlapping(&right) },
        // // SAFETY: satisfied by (i) and (ii)
        // (_, 0) => unsafe { target.copy_from_nonoverlapping(&left) },
        _ => {
            // let is_large_on_left = left.len() >= right.len();
            // if is_large_on_left != params.put_large_to_left {
            //     (left, right) = (right, left);
            // }

            // match params.streak_search {
            //     StreakSearch::None => seq_merge_streak_none(is_leq, left, right, target),
            //     StreakSearch::Linear => seq_merge_streak_linear(is_leq, left, right, target),
            //     StreakSearch::Binary => seq_merge_streak_binary(is_leq, left, right, target),
            // }
        }
    }
}

/// # SAFETY
///
/// - (i) length of `target` must equal sum of lengths of `left` and `right`.
/// - (ii) no pair of `left`, `right` and `target` can be overlapping.
/// - (iii) `left` and `right` must both have positive lengths.
pub fn seq_merge_unchecked<'a, T: 'a, F>(
    is_leq: F,
    mut left: Slice<T>,
    mut right: Slice<T>,
    target: Slice<T>,
    params: ParamsSeqMergeSortedSlices,
) where
    F: Fn(&T, &T) -> bool,
{
    // let is_large_on_left = left.len() >= right.len();
    // if is_large_on_left != params.put_large_to_left {
    //     (left, right) = (right, left);
    // }

    // match params.streak_search {
    //     StreakSearch::None => seq_merge_streak_none(is_leq, left, right, target),
    //     StreakSearch::Linear => seq_merge_streak_linear(is_leq, left, right, target),
    //     StreakSearch::Binary => seq_merge_streak_binary(is_leq, left, right, target),
    // }
}

fn seq_merge_streak_none<'a, T: 'a, F>(is_leq: F, left: Slice<T>, right: Slice<T>, target: Slice<T>)
where
    F: Fn(&T, &T) -> bool,
{
    // let mut left = left.iter_ptr_src();
    // let mut right = right.iter_ptr_src();
    // let mut dst = target.iter_ptr_dst();
    // // let mut it_dst = target.iter_as_dst();

    // loop {
    //     unsafe {
    //         let l = left.current_unchecked();
    //         let r = right.current_unchecked();

    //         match is_leq(l, r) {
    //             true => {
    //                 dst.write_one_from(&mut left);
    //                 if left.is_finished() {
    //                     dst.write_rest_from(&mut right);
    //                     break;
    //                 }
    //             }
    //             false => {
    //                 dst.write_one_from(&mut right);
    //                 if right.is_finished() {
    //                     dst.write_rest_from(&mut left);
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }
}

fn seq_merge_streak_linear<'a, T: 'a, F>(
    is_leq: F,
    left: Slice<T>,
    right: Slice<T>,
    target: Slice<T>,
) where
    F: Fn(&T, &T) -> bool,
{
    const WILL_FIND: &str = "There exists at least one element satisfying the condition";

    // let mut left = left.iter_ptr_src();
    // let mut right = right.iter_ptr_src();
    // let mut dst = target.iter_ptr_dst();

    // loop {
    //     unsafe {
    //         let l = left.current_unchecked();
    //         let r = right.current_unchecked();

    //         match is_leq(l, r) {
    //             true => {
    //                 let count = left.values().position(|x| is_leq(x, r)).expect(WILL_FIND);
    //                 dst.write_many_from(&mut left, count);
    //                 if left.is_finished() {
    //                     dst.write_rest_from(&mut right);
    //                     break;
    //                 }
    //             }
    //             false => {
    //                 let count = right.values().position(|x| is_leq(x, l)).expect(WILL_FIND);
    //                 dst.write_many_from(&mut right, count);
    //                 if right.is_finished() {
    //                     dst.write_rest_from(&mut left);
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }
}

// fn seq_merge_streak_binary<'a, T: 'a, F>(
//     is_leq: F,
//     left: Slice<T>,
//     right: Slice<T>,
//     target: Slice<T>,
// ) where
//     F: Fn(&T, &T) -> bool,
// {
//     let mut it_left = left.iter_over_ptr();
//     let mut it_right = right.iter_over_ptr();
//     let mut it_dst = target.iter_as_dst();

//     loop {
//         unsafe {
//             let l = it_left.current_unchecked();
//             let r = it_right.current_unchecked();

//             match is_leq(l, r) {
//                 true => {
//                     let ptr = it_left.peek_unchecked();
//                     let remaining = left.subslice_from(ptr).as_slice();
//                     let bin_search = remaining.binary_search_by(|x| match is_leq(x, r) {
//                         true => Ordering::Less,
//                         false => Ordering::Greater,
//                     });
//                     let idx = match bin_search {
//                         Ok(idx) => idx,
//                         Err(idx) => idx,
//                     };
//                     let src_begin = it_left.peek_unchecked();
//                     let src_end_inclusive = src_begin.add(idx - 1);
//                     it_left.jump_to(src_end_inclusive);

//                     it_dst.write_many_unchecked(src_begin, src_end_inclusive);

//                     if it_left.is_finished() {
//                         it_dst.write_remaining_from(&it_right.remaining_into_slice());
//                         break;
//                     }
//                 }
//                 false => {
//                     let ptr = it_right.peek_unchecked();
//                     let remaining = right.subslice_from(ptr).as_slice();
//                     let bin_search = remaining.binary_search_by(|x| match is_leq(x, l) {
//                         true => Ordering::Less,
//                         false => Ordering::Greater,
//                     });
//                     let idx = match bin_search {
//                         Ok(idx) => idx,
//                         Err(idx) => idx,
//                     };
//                     let src_begin = it_right.peek_unchecked();
//                     let src_end_inclusive = src_begin.add(idx - 1);
//                     it_right.jump_to(src_end_inclusive);

//                     it_dst.write_many_unchecked(src_begin, src_end_inclusive);

//                     if it_right.is_finished() {
//                         it_dst.write_remaining_from(&it_left.remaining_into_slice());
//                         break;
//                     }
//                 }
//             }
//         }
//     }
// }
