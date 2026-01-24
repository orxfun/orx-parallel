use crate::{ParIter, ParThreadPool, ParallelizableCollection};
use alloc::vec::Vec;
use core::{num::NonZeroUsize, ops::Range, ptr::slice_from_raw_parts_mut};

pub fn sort<P, T>(pool: &mut P, num_threads: NonZeroUsize, slice: &mut [T])
where
    P: ParThreadPool,
    T: Ord,
{
    let num_chunks: usize = pool.max_num_threads().min(num_threads).into();
    let ranges = slice_ranges(slice.len(), num_chunks);
    let p = slice.as_mut_ptr() as usize;
    ranges.par().for_each(|range| {
        let p = p as *mut T;
        let ptr = unsafe { p.add(range.start) };
        let slice = unsafe { &mut *slice_from_raw_parts_mut(ptr, range.len()) };
        slice.sort();
    });
    // slice.sort();
}

pub(super) fn slice_ranges(len: usize, num_chunks: usize) -> Vec<Range<usize>> {
    let num_chunks = match num_chunks > len {
        true => len,
        false => num_chunks,
    };

    let mut slices = Vec::with_capacity(num_chunks);

    match num_chunks {
        0 => {}
        _ => {
            let avg_len = len / num_chunks;
            let lower_sum = avg_len * num_chunks;
            let num_larger = len - lower_sum;

            let mut begin = 0;
            let chunk_size = |c: usize| match c < num_larger {
                true => avg_len + 1,
                false => avg_len,
            };
            for c in 0..num_chunks {
                let end = begin + chunk_size(c);
                slices.push(begin..end);
                begin = end;
            }
        }
    }

    slices
}
