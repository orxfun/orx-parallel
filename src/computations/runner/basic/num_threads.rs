use crate::parameters::NumThreads;

const MAX_UNSET_NUM_THREADS: usize = 8;

pub fn maximum_num_threads(input_len: Option<usize>, num_threads: NumThreads) -> usize {
    match num_threads {
        NumThreads::Auto => from_auto_num_threads(input_len),
        NumThreads::Max(x) => from_max_num_threads(input_len, x.into()),
    }
    .max(1)
}

fn from_auto_num_threads(input_len: Option<usize>) -> usize {
    match std::thread::available_parallelism() {
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

fn from_max_num_threads(input_len: Option<usize>, num_threads: usize) -> usize {
    match std::thread::available_parallelism() {
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
