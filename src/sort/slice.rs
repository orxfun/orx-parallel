use crate::{ParIter, ParThreadPool, ParallelizableCollection};
use alloc::vec;
use alloc::vec::Vec;
use core::{num::NonZeroUsize, ptr::slice_from_raw_parts_mut};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

#[derive(Debug, Copy, Clone)]
pub enum SortChunks {
    SeqWithPriorityQueue,
}

pub fn sort<P, T>(pool: &mut P, num_threads: NonZeroUsize, slice: &mut [T], sort_chunks: SortChunks)
where
    P: ParThreadPool,
    T: Ord,
{
    let num_chunks: usize = pool.max_num_threads().min(num_threads).into();
    assert_eq!(num_chunks, 32);

    let mut buf = Vec::<T>::with_capacity(slice.len());
    let buf_ptr = buf.as_mut_ptr();
    unsafe { buf_ptr.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) };

    let chunks = slice_chunks(buf_ptr, slice.len(), num_chunks);

    // chunks.par().for_each(|chunk| chunk.as_mut_slice().sort());
    std::thread::scope(|s| {
        for chunk in &chunks {
            s.spawn(|| chunk.as_mut_slice().sort());
        }
    });

    match sort_chunks {
        SortChunks::SeqWithPriorityQueue => sort_chunks_by_queue(chunks, slice),
    }
}

fn sort_chunks_by_queue<T>(chunks: Vec<SliceChunk<T>>, slice: &mut [T])
where
    T: Ord,
{
    let mut queue = BinaryHeap::with_capacity(chunks.len());
    let mut indices = vec![0; chunks.len()];

    for (c, chunk) in chunks.iter().enumerate() {
        if let Some(x) = chunk.get(indices[c]) {
            queue.push(c, x);
        }
    }

    let mut dst = slice.as_mut_ptr();
    let mut curr_c = queue.pop_node();
    while let Some(c) = curr_c {
        let idx = indices[c];
        indices[c] += 1;

        curr_c = match chunks[c].get(indices[c]) {
            Some(x) => Some(queue.push_then_pop(c, x).0),
            None => queue.pop_node(),
        };

        let src = unsafe { chunks[c].ptr_at(idx) };
        unsafe { dst.copy_from_nonoverlapping(src, 1) };
        dst = unsafe { dst.add(1) };
    }
}

pub(super) struct SliceChunk<T> {
    data: *mut T,
    len: usize,
}
unsafe impl<T> Send for SliceChunk<T> {}
unsafe impl<T> Sync for SliceChunk<T> {}

impl<T> SliceChunk<T> {
    pub(super) fn as_mut_slice(&self) -> &mut [T] {
        unsafe { &mut *slice_from_raw_parts_mut(self.data, self.len) }
    }

    fn get(&self, i: usize) -> Option<&T> {
        match i < self.len {
            true => Some(unsafe { &*self.data.add(i) }),
            false => None,
        }
    }

    #[inline(always)]
    unsafe fn ptr_at(&self, i: usize) -> *const T {
        unsafe { &*self.data.add(i) }
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
                let len = chunk_size(c);
                slices.push(SliceChunk { data, len });

                let end = begin + len;
                begin = end;
            }
        }
    }

    slices
}
