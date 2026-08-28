use crate::{IntoParIter, Par, ParThreadPool, Params, runner::ParRunner};
use alloc::vec::Vec;
use core::mem::MaybeUninit;

/// Task for sorting a disjoint chunk of the slice in parallel.
struct SortChunkTask<T> {
    ptr: *mut T,
    len: usize,
}
unsafe impl<T: Send> Send for SortChunkTask<T> {}
unsafe impl<T: Send> Sync for SortChunkTask<T> {}

impl<T: Ord> SortChunkTask<T> {
    #[inline]
    fn execute(self) {
        if self.len > 1 {
            let s = unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) };
            s.sort_unstable();
        }
    }
}

/// Binary searches the split point in two sorted slices A and B such that the first `r`
/// elements of merged(A, B) consist of A[0..a_split] and B[0..b_split] with a_split + b_split == r.
fn find_split<T: Ord>(a: &[T], b: &[T], r: usize) -> (usize, usize) {
    let mut low = r.saturating_sub(b.len());
    let mut high = a.len().min(r);
    while low < high {
        let mid = low + (high - low) / 2;
        let b_idx = r - mid - 1;
        match a[mid] < b[b_idx] {
            true => low = mid + 1,
            false => high = mid,
        }
    }
    (low, r - low)
}

/// Task for merging two sorted contiguous sub-slices from `src_a` and `src_b` into `dst`.
struct MergeSubTask<T> {
    src_a: *const T,
    len_a: usize,
    src_b: *const T,
    len_b: usize,
    dst: *mut T,
}
unsafe impl<T: Send> Send for MergeSubTask<T> {}
unsafe impl<T: Send> Sync for MergeSubTask<T> {}

impl<T: Ord> MergeSubTask<T> {
    #[inline]
    fn execute(self) {
        if self.len_a == 0 {
            if self.len_b > 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(self.src_b, self.dst, self.len_b);
                }
            }
            return;
        }
        if self.len_b == 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(self.src_a, self.dst, self.len_a);
            }
            return;
        }

        // Fast-path: if all elements of A <= all elements of B
        let last_a = unsafe { &*self.src_a.add(self.len_a - 1) };
        let first_b = unsafe { &*self.src_b };
        if last_a <= first_b {
            unsafe {
                core::ptr::copy_nonoverlapping(self.src_a, self.dst, self.len_a);
                core::ptr::copy_nonoverlapping(self.src_b, self.dst.add(self.len_a), self.len_b);
            }
            return;
        }

        // Fast-path: if all elements of B < all elements of A
        let last_b = unsafe { &*self.src_b.add(self.len_b - 1) };
        let first_a = unsafe { &*self.src_a };
        if last_b < first_a {
            unsafe {
                core::ptr::copy_nonoverlapping(self.src_b, self.dst, self.len_b);
                core::ptr::copy_nonoverlapping(self.src_a, self.dst.add(self.len_b), self.len_a);
            }
            return;
        }

        // Standard merge
        unsafe {
            let mut ptr_a = self.src_a;
            let end_a = self.src_a.add(self.len_a);
            let mut ptr_b = self.src_b;
            let end_b = self.src_b.add(self.len_b);
            let mut out = self.dst;

            while ptr_a < end_a && ptr_b < end_b {
                match *ptr_b < *ptr_a {
                    true => {
                        core::ptr::copy_nonoverlapping(ptr_b, out, 1);
                        ptr_b = ptr_b.add(1);
                    }
                    false => {
                        core::ptr::copy_nonoverlapping(ptr_a, out, 1);
                        ptr_a = ptr_a.add(1);
                    }
                }
                out = out.add(1);
            }

            if ptr_a < end_a {
                let rem = end_a.offset_from(ptr_a) as usize;
                core::ptr::copy_nonoverlapping(ptr_a, out, rem);
            } else if ptr_b < end_b {
                let rem = end_b.offset_from(ptr_b) as usize;
                core::ptr::copy_nonoverlapping(ptr_b, out, rem);
            }
        }
    }
}

/// Task for parallel copying back to original slice if needed.
struct CopyChunkTask<T> {
    src: *const T,
    dst: *mut T,
    len: usize,
}
unsafe impl<T: Send> Send for CopyChunkTask<T> {}
unsafe impl<T: Send> Sync for CopyChunkTask<T> {}

impl<T> CopyChunkTask<T> {
    #[inline]
    fn execute(self) {
        if self.len > 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(self.src, self.dst, self.len);
            }
        }
    }
}

/// Sorts the `slice` in parallel using the provided `runner` and parallelization `params`.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let mut data = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
/// let mut runner = Runner::fixed();
/// par_experimental_sort(&mut data, &mut runner, Params::default());
/// assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
/// ```
pub fn par_experimental_sort<T, R>(slice: &mut [T], runner: &mut R, params: Params)
where
    T: Ord + Send,
    R: ParRunner,
{
    let n = slice.len();
    if n <= 1 {
        return;
    }

    if params.is_sequential() {
        slice.sort_unstable();
        return;
    }

    let max_threads = runner
        .pool()
        .max_num_threads_for_computation(params, (n, Some(n)));

    if max_threads <= 1 || n < 1024 {
        slice.sort_unstable();
        return;
    }

    // Determine number of initial chunks K (must be a power of 2)
    let mut num_chunks = (max_threads * 2).next_power_of_two();
    while num_chunks > 2 && n / num_chunks < 512 {
        num_chunks /= 2;
    }

    if n / num_chunks < 64 {
        slice.sort_unstable();
        return;
    }

    let orig_ptr = slice.as_mut_ptr();

    // Phase 1: Sort each chunk in parallel
    let mut sort_tasks = Vec::with_capacity(num_chunks);
    for i in 0..num_chunks {
        let start = i * n / num_chunks;
        let end = (i + 1) * n / num_chunks;
        let len = end - start;
        sort_tasks.push(SortChunkTask {
            ptr: unsafe { orig_ptr.add(start) },
            len,
        });
    }

    let par = sort_tasks.into_par().runner(&mut *runner);
    let par = params.apply(par);
    par.for_each(|task| task.execute());

    // Allocate auxiliary buffer for merging
    let mut aux = Vec::<MaybeUninit<T>>::with_capacity(n);
    let aux_ptr = aux.as_mut_ptr() as *mut T;

    // Phase 2: Hierarchical merge passes
    let mut current_chunks = num_chunks;
    let mut pass = 0;

    while current_chunks > 1 {
        let (src_base, dst_base) = if pass % 2 == 0 {
            (orig_ptr, aux_ptr)
        } else {
            (aux_ptr, orig_ptr)
        };

        let next_chunks = current_chunks / 2;
        let stride = num_chunks / current_chunks;
        let tasks_per_pair = (max_threads / next_chunks).max(1);
        let mut merge_tasks = Vec::with_capacity(next_chunks * tasks_per_pair);

        for pair_idx in 0..next_chunks {
            let chunk_a_idx = pair_idx * 2;
            let chunk_b_idx = chunk_a_idx + 1;

            let start_a = (chunk_a_idx * stride) * n / num_chunks;
            let end_a = ((chunk_a_idx + 1) * stride) * n / num_chunks;
            let len_a = end_a - start_a;

            let start_b = (chunk_b_idx * stride) * n / num_chunks;
            let end_b = ((chunk_b_idx + 1) * stride) * n / num_chunks;
            let len_b = end_b - start_b;

            let slice_a = unsafe { core::slice::from_raw_parts(src_base.add(start_a), len_a) };
            let slice_b = unsafe { core::slice::from_raw_parts(src_base.add(start_b), len_b) };

            let pair_total = len_a + len_b;

            let mut prev_a = 0;
            let mut prev_b = 0;
            let mut prev_r = 0;

            for t in 1..=tasks_per_pair {
                let r = t * pair_total / tasks_per_pair;
                let (curr_a, curr_b) = if t == tasks_per_pair {
                    (len_a, len_b)
                } else {
                    find_split(slice_a, slice_b, r)
                };

                let sub_len_a = curr_a - prev_a;
                let sub_len_b = curr_b - prev_b;
                let sub_dst = unsafe { dst_base.add(start_a + prev_r) };

                merge_tasks.push(MergeSubTask {
                    src_a: unsafe { slice_a.as_ptr().add(prev_a) },
                    len_a: sub_len_a,
                    src_b: unsafe { slice_b.as_ptr().add(prev_b) },
                    len_b: sub_len_b,
                    dst: sub_dst,
                });

                prev_a = curr_a;
                prev_b = curr_b;
                prev_r = r;
            }
        }

        let par = merge_tasks.into_par().runner(&mut *runner);
        let par = params.apply(par);
        par.for_each(|task| task.execute());

        current_chunks = next_chunks;
        pass += 1;
    }

    // If odd number of passes, the sorted data is in `aux_ptr`, copy back to `orig_ptr`
    if pass % 2 != 0 {
        let copy_chunks = max_threads.min(n);
        let mut copy_tasks = Vec::with_capacity(copy_chunks);
        for i in 0..copy_chunks {
            let start = i * n / copy_chunks;
            let end = (i + 1) * n / copy_chunks;
            let len = end - start;
            copy_tasks.push(CopyChunkTask {
                src: unsafe { aux_ptr.add(start) },
                dst: unsafe { orig_ptr.add(start) },
                len,
            });
        }

        let par = copy_tasks.into_par().runner(&mut *runner);
        let par = params.apply(par);
        par.for_each(|task| task.execute());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::default_runner;
    use alloc::string::String;

    #[test]
    fn test_sort_empty_and_single() {
        let mut runner = default_runner();
        let mut empty: [i32; 0] = [];
        par_experimental_sort(&mut empty, &mut runner, Params::default());

        let mut single = [42];
        par_experimental_sort(&mut single, &mut runner, Params::default());
        assert_eq!(single, [42]);
    }

    #[test]
    fn test_sort_small_slices() {
        let mut runner = default_runner();
        let mut data = [9, 3, 7, 1, 5, 2, 8, 4, 6];
        par_experimental_sort(&mut data, &mut runner, Params::default());
        assert_eq!(data, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_sort_medium_and_large_random() {
        let mut runner = default_runner();

        for size in [500, 1024, 2048, 5000, 20000, 50000] {
            let mut data: Vec<i32> = (0..size as u64)
                .map(|i| (i.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFFFFFF) as i32)
                .collect();
            let mut expected = data.clone();
            expected.sort_unstable();

            par_experimental_sort(&mut data, &mut runner, Params::default());
            assert_eq!(data, expected, "Failed for size {}", size);
        }
    }

    #[test]
    fn test_sort_sorted_and_reversed() {
        let mut runner = default_runner();
        let size = 10000;

        let mut sorted: Vec<i32> = (0..size).collect();
        par_experimental_sort(&mut sorted, &mut runner, Params::default());
        assert!(sorted.windows(2).all(|w| w[0] <= w[1]));

        let mut reversed: Vec<i32> = (0..size).rev().collect();
        par_experimental_sort(&mut reversed, &mut runner, Params::default());
        assert!(reversed.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_sort_high_duplicates() {
        let mut runner = default_runner();
        let size = 20000;
        let mut data: Vec<i32> = (0..size).map(|i| i % 7).collect();
        let mut expected = data.clone();
        expected.sort_unstable();

        par_experimental_sort(&mut data, &mut runner, Params::default());
        assert_eq!(data, expected);
    }

    #[test]
    fn test_sort_non_copy_types() {
        let mut runner = default_runner();
        let size = 5000;
        let mut data: Vec<String> = (0..size)
            .map(|i| alloc::format!("item_{:06}", (i * 7919) % size))
            .collect();
        let mut expected = data.clone();
        expected.sort();

        par_experimental_sort(&mut data, &mut runner, Params::default());
        assert_eq!(data, expected);
    }

    #[test]
    fn test_sort_num_threads_configs() {
        for nt in [1, 2, 4, 8] {
            let mut runner = default_runner();
            let size = 10000;
            let mut data: Vec<i32> = (0..size as u64)
                .map(|i| (i.wrapping_mul(2654435761) & 0x7FFFFFFF) as i32)
                .collect();
            let mut expected = data.clone();
            expected.sort_unstable();

            let params = Params::default().with_num_threads(nt);
            par_experimental_sort(&mut data, &mut runner, params);
            assert_eq!(data, expected, "Failed for num_threads = {}", nt);
        }
    }
}
