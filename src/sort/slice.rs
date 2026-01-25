use crate::{ParIter, ParThreadPool, ParallelizableCollection};
use alloc::vec;
use alloc::vec::Vec;
use core::{num::NonZeroUsize, ops::Range, ptr::slice_from_raw_parts_mut};
use orx_priority_queue::BinaryHeap;

pub fn sort<P, T>(pool: &mut P, num_threads: NonZeroUsize, slice: &mut [T])
where
    P: ParThreadPool,
    T: Ord,
{
    let num_chunks: usize = pool.max_num_threads().min(num_threads).into();
    // let ranges = slice_ranges(slice.len(), num_chunks);

    let mut buf = Vec::<T>::with_capacity(slice.len());
    let buf_ptr = buf.as_mut_ptr();
    unsafe { buf_ptr.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) };

    let chunks = slice_chunks(slice.as_mut_ptr(), slice.len(), num_chunks);

    chunks.par().for_each(|chunk| chunk.as_mut_slice().sort());

    // let mut queue = BinaryHeap::with_capacity(ranges.len());
    // let mut indices = vec![0; ranges.len()];

    // for (v, vec) in ranges.iter().enumerate() {
    //     if let Some(x) = vec.get(indices[v])
    //         && x.0 <= max_idx_or_inf
    //     {
    //         queue.push(v, x.0);
    //     }
    // }
    // let mut curr_v = queue.pop_node();
    // slice.sort();
}

pub(super) struct SliceChunk<T>(*mut T, usize);
unsafe impl<T> Send for SliceChunk<T> {}
unsafe impl<T> Sync for SliceChunk<T> {}

impl<T> SliceChunk<T> {
    pub(super) fn as_mut_slice(&self) -> &mut [T] {
        unsafe { &mut *slice_from_raw_parts_mut(self.0, self.1) }
    }
}

pub(super) fn slice_chunks<T>(data: *mut T, len: usize, num_chunks: usize) -> Vec<SliceChunk<T>> {
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
                let data = unsafe { data.add(begin) };
                let chunk_size = chunk_size(c);
                slices.push(SliceChunk(data, chunk_size));

                let end = begin + chunk_size;
                begin = end;
            }
        }
    }

    slices
}
