use crate::sort::slice_chunks::Slice;
use orx_iterable::Iterable;

#[derive(Debug, Clone, Copy)]
pub enum MergeSliceKind {
    Sequential,
    Parallel { nt: usize },
}

pub fn merge_slices<T: Ord>(kind: MergeSliceKind, left: Slice<T>, right: Slice<T>, dst: Slice<T>) {
    match kind {
        MergeSliceKind::Sequential => sequential(left, right, dst),
        MergeSliceKind::Parallel { nt } => parallel(left, right, dst, nt),
    }
}

fn sequential<T: Ord>(left: Slice<T>, right: Slice<T>, dst: Slice<T>) {
    let mut left = left.iter_ptr();
    let mut right = right.iter_ptr();
    let mut dst = dst.into_dst();
    match (left.len(), right.len()) {
        (0, 0) => {}
        (0, _) => dst.write_remaining_from(&right),
        (_, 0) => dst.write_remaining_from(&left),
        _ => loop {
            let (a, b) = match unsafe { left.peek_value_unchecked() < right.peek_value_unchecked() }
            {
                true => (&mut left, &mut right),
                false => (&mut right, &mut left),
            };

            dst.write_one(unsafe { a.next_unchecked() });
            if a.is_finished() {
                dst.write_remaining_from(b);
                break;
            }
        },
    }
}

fn parallel<T: Ord>(left: Slice<T>, right: Slice<T>, dst: Slice<T>, nt: usize) {
    let left_mid = left.len / 2;
    let right_mid = find_right_mid_index(&left[left_mid], &right);
    //
}

fn find_right_mid_index<T: Ord>(pivot_value: &T, right: &Slice<T>) -> usize {
    right
        .iter()
        .position(|x| x > pivot_value)
        .unwrap_or(right.len())
}
