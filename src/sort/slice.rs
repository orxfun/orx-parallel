use crate::NumThreads;

pub fn sort<T: Ord>(slice: &mut [T], num_threads: NumThreads) {
    slice.sort();
}
