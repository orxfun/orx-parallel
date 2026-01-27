use crate::sort::slice_chunk::SliceChunk;

#[derive(Debug, Clone, Copy)]
pub enum MergeSliceKind {
    Sequential,
    Parallel { nt: usize },
}

pub fn merge_slices<T: Ord>(
    kind: MergeSliceKind,
    left: SliceChunk<T>,
    right: SliceChunk<T>,
    dst: SliceChunk<T>,
) {
    match kind {
        MergeSliceKind::Sequential => sequential(left, right, dst),
        MergeSliceKind::Parallel { nt } => parallel(left, right, dst, nt),
    }
}

fn sequential<T: Ord>(left: SliceChunk<T>, right: SliceChunk<T>, dst: SliceChunk<T>) {
    let mut left_ptr = left.data;
    let mut right_ptr = right.data;
    let inc_end_left = unsafe { left_ptr.add(left.len - 1) };
    let inc_end_right = unsafe { right_ptr.add(right.len - 1) };
    let mut q = dst.data;
    for _ in 0..dst.len {
        let (less_ptr, less_ptr_end, more_ptr, more_end) = match unsafe { &*left_ptr < &*right_ptr }
        {
            true => (&mut left_ptr, inc_end_left, right_ptr, inc_end_right),
            false => (&mut right_ptr, inc_end_right, left_ptr, inc_end_left),
        };

        unsafe { q.copy_from_nonoverlapping(*less_ptr, 1) };
        match *less_ptr == less_ptr_end {
            true => {
                let remaining = unsafe { more_end.offset_from(more_ptr) } as usize + 1;
                unsafe { q = q.add(1) };
                unsafe { q.copy_from_nonoverlapping(more_ptr, remaining) };
                break;
            }
            false => {
                unsafe { *less_ptr = less_ptr.add(1) };
                unsafe { q = q.add(1) };
            }
        }
    }
}

fn parallel<T: Ord>(left: SliceChunk<T>, right: SliceChunk<T>, dst: SliceChunk<T>, nt: usize) {
    let left_mid = left.len / 2;

    //
}

fn find_right_mid_index<T: Ord>(pivot_value: &T, right: SliceChunk<T>, dst: SliceChunk<T>) {
    // for (i, value) in right
}
