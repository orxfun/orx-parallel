use crate::{
    ParIter, ParThreadPool, Parallelizable, ParallelizableCollection, sort::slice_chunk::SliceChunk,
};
use alloc::vec;
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use orx_priority_queue::{BinaryHeap, PriorityQueue};

pub fn sort3<T>(slice: &mut [T], depth: usize)
where
    T: Ord,
{
    let n = slice.len();
    let num_chunks = 1 << depth;

    let mut b = Vec::<T>::with_capacity(n);

    let chunks_a = SliceChunk::slice_chunks(slice.as_mut_ptr(), n, num_chunks);
    let chunks_b = SliceChunk::slice_chunks(b.as_mut_ptr(), n, num_chunks);

    // let start = std::time::Instant::now();
    chunks_a.par().for_each(|chunk| chunk.as_mut_slice().sort());
    // std::println!("elapsed = {:?}", start.elapsed());

    // let start = std::time::Instant::now();
    for d in (1..=depth).rev() {
        let num_merges = 1 << (d - 1);
        (0..num_merges).par().for_each(|m| {
            merge(&chunks_a, &chunks_b, depth, d, m);
        });
        // for m in 0..num_merges {
        //     merge(&chunks_a, &chunks_b, depth, d, m);
        // }
    }
    // std::println!("elapsed = {:?}", start.elapsed());
}

// ####################

pub fn sort2<T>(slice: &mut [T], depth: usize)
where
    T: Ord,
{
    let n = slice.len();
    let num_chunks = 1 << depth;

    let mut b = Vec::<T>::with_capacity(n);

    let chunks_a = SliceChunk::slice_chunks(slice.as_mut_ptr(), n, num_chunks);
    let chunks_b = SliceChunk::slice_chunks(b.as_mut_ptr(), n, num_chunks);

    // let start = std::time::Instant::now();
    chunks_a.par().for_each(|chunk| chunk.as_mut_slice().sort());
    // std::println!("elapsed = {:?}", start.elapsed());

    // let start = std::time::Instant::now();
    for d in (1..=depth).rev() {
        let num_merges = 1 << (d - 1);
        (0..num_merges).par().for_each(|m| {
            merge(&chunks_a, &chunks_b, depth, d, m);
        });
        // for m in 0..num_merges {
        //     merge(&chunks_a, &chunks_b, depth, d, m);
        // }
    }
    // std::println!("elapsed = {:?}", start.elapsed());
}

fn merge<T>(
    chunks_a: &[SliceChunk<T>],
    chunks_b: &[SliceChunk<T>],
    depth: usize,
    d: usize,
    m: usize,
) where
    T: Ord,
{
    let inc = 1 << (depth - d);
    let beg1 = m * 2 * inc;
    let end1 = beg1 + inc;
    let beg2 = end1;
    let end2 = beg2 + inc;

    let (chunks_src, chunks_dst) = match d.is_multiple_of(2) {
        true => (chunks_a, chunks_b),
        false => (chunks_b, chunks_a),
    };

    let src1 = SliceChunk::merged_slice(&chunks_src[beg1..end1]);
    let src2 = SliceChunk::merged_slice(&chunks_src[beg2..end2]);
    let dst = SliceChunk::merged_slice(&chunks_dst[beg1..end2]);
    debug_assert_eq!(src1.len + src2.len, dst.len);

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

// ####################

#[derive(Debug, Copy, Clone)]
pub enum SortChunks {
    SeqWithPriorityQueue,
    SeqWithPriorityQueuePtrs,
    SeqWithVec,
}

pub fn sort<P, T>(pool: &mut P, num_threads: NonZeroUsize, slice: &mut [T], sort_chunks: SortChunks)
where
    P: ParThreadPool,
    T: Ord,
{
    let num_chunks: usize = pool.max_num_threads().min(num_threads).into();
    // assert_eq!(num_chunks, 32);

    let mut buf = Vec::<T>::with_capacity(slice.len());
    let buf_ptr = buf.as_mut_ptr();
    unsafe { buf_ptr.copy_from_nonoverlapping(slice.as_ptr(), slice.len()) };

    let chunks = SliceChunk::slice_chunks(buf_ptr, slice.len(), num_chunks);

    // chunks.par().for_each(|chunk| chunk.as_mut_slice().sort());
    // let start = std::time::Instant::now();
    std::thread::scope(|s| {
        for chunk in &chunks {
            s.spawn(|| chunk.as_mut_slice().sort());
        }
    });
    // let elapsed = start.elapsed();
    // std::println!("elapsed = {elapsed:?}");
    // let start = std::time::Instant::now();

    match sort_chunks {
        SortChunks::SeqWithPriorityQueue => sort_chunks_by_queue(chunks, slice),
        SortChunks::SeqWithPriorityQueuePtrs => sort_chunks_by_queue_ptrs(chunks, slice),
        SortChunks::SeqWithVec => sort_chunks_by_vec(chunks, slice),
    }
    // let elapsed = start.elapsed();
    // std::println!("elapsed = {elapsed:?}");
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

fn sort_chunks_by_queue_ptrs<T>(chunks: Vec<SliceChunk<T>>, slice: &mut [T])
where
    T: Ord,
{
    let mut queue = BinaryHeap::with_capacity(chunks.len());
    let mut ptrs_counts: Vec<_> = chunks.iter().map(|c| (unsafe { c.ptr_at(0) }, 1)).collect();

    for (c, _) in chunks.iter().enumerate() {
        queue.push(c, unsafe { &*ptrs_counts[c].0 });
    }

    let mut dst = slice.as_mut_ptr();
    let mut curr_c = queue.pop_node();
    while let Some(c) = curr_c {
        let (src, count) = ptrs_counts[c];

        curr_c = match count < chunks[c].len {
            true => {
                unsafe { ptrs_counts[c].0 = ptrs_counts[c].0.add(1) };
                ptrs_counts[c].1 += 1;
                Some(queue.push_then_pop(c, unsafe { &*ptrs_counts[c].0 }).0)
            }
            false => queue.pop_node(),
        };

        unsafe { dst.copy_from_nonoverlapping(src, 1) };
        dst = unsafe { dst.add(1) };
    }
}

fn sort_chunks_by_vec<T>(chunks: Vec<SliceChunk<T>>, slice: &mut [T])
where
    T: Ord,
{
    let mut indices = vec![0; chunks.len()];

    for dst in slice.iter_mut() {
        let mut min_c = usize::MAX;
        let mut curr = None;
        for (c, chunk) in chunks.iter().enumerate() {
            match chunk.get(indices[c]) {
                Some(x) => match curr {
                    Some(y) if x < y => {
                        curr = Some(x);
                        min_c = c;
                    }
                    None => {
                        curr = Some(x);
                        min_c = c;
                    }
                    _ => {}
                },
                None => {}
            }
        }

        let idx = indices[min_c];
        indices[min_c] += 1;
        let src = unsafe { chunks[min_c].ptr_at(idx) };
        let dst = dst as *mut T;
        unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    let mut queue = BinaryHeap::with_capacity(chunks.len());
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
