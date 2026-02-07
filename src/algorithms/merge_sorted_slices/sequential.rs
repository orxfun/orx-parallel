use crate::algorithms::data_structures::{Slice, SliceMut};
use crate::algorithms::merge_sorted_slices::alg::{MergeSortedSlicesParams, StreakSearch};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::Range;

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

    match params.sequential_merge_threshold {
        0 => match params.streak_search {
            StreakSearch::None => merge_sorted_slices_streak_none(is_leq, left, right, target),
            StreakSearch::Linear => {
                merge_sorted_slices_with_streak_linear(is_leq, left, right, target)
            }
            StreakSearch::Binary => {
                merge_sorted_slices_with_streak_binary(is_leq, left, right, target)
            }
        },
        sequential_merge_threshold => {
            merge_sorted_slices_by_dividing(
                is_leq,
                left.clone(),
                right.clone(),
                target,
                sequential_merge_threshold,
            );
        }
    }
}

fn merge_sorted_slices_streak_none<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    match (left.len(), right.len()) {
        (0, _) => unsafe { target.iter_as_dst().write_remaining_from(right) },
        (_, 0) => unsafe { target.iter_as_dst().write_remaining_from(left) },
        _ => {
            let mut left = left.iter_over_ptr();
            let mut right = right.iter_over_ptr();
            let mut dst = target.iter_as_dst();

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
    }
}

fn merge_sorted_slices_with_streak_linear<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    match (left.len(), right.len()) {
        (0, _) => unsafe { target.iter_as_dst().write_remaining_from(right) },
        (_, 0) => unsafe { target.iter_as_dst().write_remaining_from(left) },
        _ => {
            let mut left = left.iter_over_ptr();
            let mut right = right.iter_over_ptr();
            let mut dst = target.iter_as_dst();

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
    }
}

fn merge_sorted_slices_with_streak_binary<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    match (left.len(), right.len()) {
        (0, _) => unsafe { target.iter_as_dst().write_remaining_from(right) },
        (_, 0) => unsafe { target.iter_as_dst().write_remaining_from(left) },
        _ => {
            let mut left_it = left.iter_over_ptr();
            let mut right_it = right.iter_over_ptr();
            let mut dst = target.iter_as_dst();

            loop {
                let l = unsafe { left_it.current_unchecked() };
                let r = unsafe { right_it.current_unchecked() };

                match is_leq(l, r) {
                    true => {
                        let left_remaining =
                            left.subslice_from(left_it.peek_unchecked()).as_slice();
                        let bin_search = left_remaining.binary_search_by(|x| match is_leq(x, r) {
                            true => Ordering::Less,
                            false => Ordering::Greater,
                        });
                        let idx = match bin_search {
                            Ok(idx) => idx,
                            Err(idx) => idx,
                        };
                        let src_begin = left_it.peek_unchecked();
                        let src_end_inclusive = unsafe { src_begin.add(idx - 1) };
                        unsafe { left_it.jump_to(src_end_inclusive) };

                        unsafe { dst.write_many_unchecked(src_begin, src_end_inclusive) };

                        if left_it.is_finished() {
                            unsafe { dst.write_remaining_from(&right_it.remaining_into_slice()) };
                            break;
                        }
                    }
                    false => {
                        let right_remaining =
                            right.subslice_from(right_it.peek_unchecked()).as_slice();
                        let bin_search = right_remaining.binary_search_by(|x| match is_leq(x, l) {
                            true => Ordering::Less,
                            false => Ordering::Greater,
                        });
                        let idx = match bin_search {
                            Ok(idx) => idx,
                            Err(idx) => idx,
                        };
                        let src_begin = right_it.peek_unchecked();
                        let src_end_inclusive = unsafe { src_begin.add(idx - 1) };
                        unsafe { right_it.jump_to(src_end_inclusive) };

                        unsafe { dst.write_many_unchecked(src_begin, src_end_inclusive) };

                        if right_it.is_finished() {
                            unsafe { dst.write_remaining_from(&left_it.remaining_into_slice()) };
                            break;
                        }
                    }
                }
            }
        }
    }
}

// divide & conquer

struct Task<'a, T: 'a> {
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target_range: Range<usize>,
}

impl<'a, T: 'a> Task<'a, T> {
    fn new(left: Slice<'a, T>, right: Slice<'a, T>, target_range: Range<usize>) -> Self {
        Self {
            left,
            right,
            target_range,
        }
    }

    fn do_sequentially(&self, sequential_merge_threshold: usize) -> bool {
        self.left.len() < 2
            || self.right.len() < 2
            || self.left.len() + self.right.len() <= sequential_merge_threshold
    }
}

struct TaskQueue<'a, T: 'a> {
    queue: Vec<Task<'a, T>>,
}

impl<'a, T: 'a> TaskQueue<'a, T> {
    fn new(left: Slice<'a, T>, right: Slice<'a, T>) -> Self {
        let mut queue = Vec::new();
        let range = 0..(left.len() + right.len());
        queue.push(Task::new(left, right, range));
        Self { queue }
    }

    fn pop(&mut self) -> Option<Task<'a, T>> {
        self.queue.pop()
    }

    fn push(&mut self, left: Task<'a, T>, right: Task<'a, T>) {
        self.queue.push(left);
        self.queue.push(right);
    }
}

fn merge_sorted_slices_by_dividing<'a, T: 'a, F>(
    is_leq: F,
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
    sequential_merge_threshold: usize,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut queue = TaskQueue::new(left, right);

    while let Some(task) = queue.pop() {
        match task.do_sequentially(sequential_merge_threshold) {
            true => {
                let mut t = target.slice(task.target_range);
                merge_sorted_slices_with_streak_linear(&is_leq, &task.left, &task.right, &mut t);
            }
            false => {
                let [left_left, left_right] = task.left.split_at_mid();
                let pivot = left_left.last().expect("left_left is not empty");
                // TODO: we could also do this with binary search!
                let right_split_at = task.right.iter_over_ref().position(|x| is_leq(pivot, x));
                let right_split_at = right_split_at.unwrap_or(task.right.len());
                let [right_left, right_right] = task.right.split_at(right_split_at);

                let begin = task.target_range.start;
                let end = begin + left_left.len() + right_left.len();
                let left_task = Task::new(left_left, right_left, begin..end);

                let begin = end;
                let end = begin + left_right.len() + right_right.len();
                let right_task = Task::new(left_right, right_right, begin..end);

                queue.push(left_task, right_task);
            }
        }
    }
}

// impl<'a, T: 'a> Task<'a, T> {
//     fn new(left: Slice<'a, T>, right: Slice<'a, T>, range: Range<usize>) -> Self {
//         Self { left, right, range }
//     }

//     fn clone(&self) -> Self {
//         Self::new(self.left.clone(), self.right.clone(), self.range.clone())
//     }
// }

// fn merge_sorted_slices_by_dividing<'a, T: 'a, F>(
//     is_leq: F,
//     left: &Slice<'a, T>,
//     right: &Slice<'a, T>,
//     target: &mut SliceMut<'a, T>,
//     sequential_merge_threshold: usize,
// ) where
//     F: Fn(&T, &T) -> bool,
// {
//     let mut stack = Vec::new();
//     stack.push(Task::new(
//         left.clone(),
//         right.clone(),
//         0..(left.len() + right.len()),
//     ));

//     loop {
//         match stack.pop() {
//             Some(task) => merge_sorted_slices_by_dividing_handler(
//                 &is_leq,
//                 task,
//                 target,
//                 sequential_merge_threshold,
//                 &mut stack,
//             ),
//             None => break,
//         }
//     }
// }

// fn merge_sorted_slices_by_dividing_handler<'a, T: 'a, F>(
//     is_leq: F,
//     task: Task<'a, T>,
//     target: &mut SliceMut<'a, T>,
//     sequential_merge_threshold: usize,
//     stack: &mut Vec<Task<'a, T>>,
// ) where
//     F: Fn(&T, &T) -> bool,
// {
//     match task.left.len() < 2
//         || task.right.len() < 2
//         || task.left.len() + task.right.len() <= sequential_merge_threshold
//     {
//         true => merge_sorted_slices_with_streak_linear(is_leq, &task.left, &task.right, target),
//         false => {
//             let [left_left, left_right] = left.split_at_mid();
//             let pivot = left_left.last().expect("left_left is not empty");
//             // TODO: we could also do this with binary search!
//             let right_split_at = right.iter_over_ref().position(|x| is_leq(pivot, x));
//             let right_split_at = right_split_at.unwrap_or(right.len());
//             let [right_left, right_right] = right.split_at(right_split_at);
//             stack.push([left_left, right_left]);
//             stack.push([left_right, right_right]);
//         }
//     }
// }
