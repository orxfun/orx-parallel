use crate::ParThreadPool;
use alloc::vec::Vec;
use core::ops::Range;

pub fn sort<P, T>(pool: &mut P, slice: &mut [T])
where
    P: ParThreadPool,
    T: Ord,
{
    slice.sort();
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
