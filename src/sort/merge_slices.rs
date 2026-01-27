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
    let mut dst = dst.into_dst();
    match (left.len(), right.len()) {
        (0, 0) => {}
        (0, _) => dst.write_remaining_from(&right.iter_ptr()),
        (_, 0) => dst.write_remaining_from(&left.iter_ptr()),
        _ => {
            let mut left = left.iter_ptr();
            let mut right = right.iter_ptr();
            loop {
                let (a, b) =
                    match unsafe { left.peek_value_unchecked() < right.peek_value_unchecked() } {
                        true => (&mut left, &mut right),
                        false => (&mut right, &mut left),
                    };

                dst.write_one(unsafe { a.next_unchecked() });
                if a.is_finished() {
                    dst.write_remaining_from(b);
                    break;
                }
            }
        }
    }
}

fn parallel<T: Ord>(left: Slice<T>, right: Slice<T>, dst: Slice<T>, nt: usize) {
    match (left.len(), right.len()) {
        (0 | 1, _) => sequential(left, right, dst),
        (_, 0 | 1) => sequential(left, right, dst),
        _ => {
            let left_mid = left.len / 2;
            let pivot = &left[left_mid];
            let right_mid = (&right)
                .iter()
                .position(|r| r > pivot)
                .unwrap_or(right.len());

            let [left_left, left_right] = left.split_at(left_mid);
            let [right_left, right_right] = right.split_at(right_mid);
            let [dst_left, dst_right] = dst.split_at(left_left.len() + right_left.len());

            parallel(left_left, right_left, dst_left, nt);
            parallel(left_right, right_right, dst_right, nt);
        }
    }
}
