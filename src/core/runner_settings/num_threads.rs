use crate::num_threads::NumThreads;
use std::num::NonZeroUsize;

pub fn calc_num_threads(input_len: Option<usize>, num_threads: NumThreads) -> usize {
    let available_threads = std::thread::available_parallelism();
    match num_threads {
        NumThreads::Auto => auto_num_threads(input_len, available_threads),
        NumThreads::Max(x) => set_num_threads(input_len, available_threads, x.into()),
    }
}

const MAX_UNSET_NUM_THREADS: usize = 8;

fn set_num_threads(
    input_len: Option<usize>,
    available_threads: Result<NonZeroUsize, std::io::Error>,
    num_threads: usize,
) -> usize {
    match available_threads {
        Err(e) => {
            debug_assert!(false, "Failed to get maximum available parallelism (std::thread::available_parallelism()); falling back to sequential execution.: {}", e);
            input_len.unwrap_or(num_threads).min(num_threads)
        }
        Ok(available_threads) => input_len
            .unwrap_or(usize::MAX)
            .min(num_threads)
            .min(available_threads.into()),
    }
}

fn auto_num_threads(
    input_len: Option<usize>,
    available_threads: Result<NonZeroUsize, std::io::Error>,
) -> usize {
    match available_threads {
        Err(e) => {
            debug_assert!(false, "Failed to get maximum available parallelism (std::thread::available_parallelism()): {}", e);
            input_len
                .unwrap_or(MAX_UNSET_NUM_THREADS)
                .min(MAX_UNSET_NUM_THREADS)
        }
        Ok(available_threads) => input_len
            .unwrap_or(MAX_UNSET_NUM_THREADS)
            .min(available_threads.into()),
    }
}
