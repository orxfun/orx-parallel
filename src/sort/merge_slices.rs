use crate::sort::slice_chunk::SliceChunk;

#[derive(Debug, Clone, Copy)]
pub enum MergeSliceKind {
    Sequential,
}

pub fn merge_slices<T: Ord>(
    kind: MergeSliceKind,
    src1: SliceChunk<T>,
    src2: SliceChunk<T>,
    dst: SliceChunk<T>,
) {
    match kind {
        MergeSliceKind::Sequential => sequential(src1, src2, dst),
    }
}

fn sequential<T: Ord>(src1: SliceChunk<T>, src2: SliceChunk<T>, dst: SliceChunk<T>) {
    let mut p1 = src1.data;
    let mut p2 = src2.data;
    let inc_end1 = unsafe { p1.add(src1.len - 1) };
    let inc_end2 = unsafe { p2.add(src2.len - 1) };
    let mut q = dst.data;
    for _ in 0..dst.len {
        match unsafe { &*p1 < &*p2 } {
            true => {
                unsafe { q.copy_from_nonoverlapping(p1, 1) };
                match p1 == inc_end1 {
                    true => {
                        let remaining2 = unsafe { inc_end2.offset_from(p2) } as usize + 1;
                        unsafe { q = q.add(1) };
                        unsafe { q.copy_from_nonoverlapping(p2, remaining2) };
                        break;
                    }
                    false => {
                        unsafe { p1 = p1.add(1) };
                        unsafe { q = q.add(1) };
                    }
                }
            }
            false => {
                unsafe { q.copy_from_nonoverlapping(p2, 1) };
                match p2 == inc_end2 {
                    true => {
                        let remaining1 = unsafe { inc_end1.offset_from(p1) } as usize + 1;
                        unsafe { q = q.add(1) };
                        unsafe { q.copy_from_nonoverlapping(p1, remaining1) };
                        break;
                    }
                    false => {
                        unsafe { p2 = p2.add(1) };
                        unsafe { q = q.add(1) };
                    }
                }
            }
        }
    }
}
