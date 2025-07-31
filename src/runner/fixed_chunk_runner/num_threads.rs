use crate::parameters::NumThreads;

const MAX_UNSET_NUM_THREADS: usize = 8;
// A: what should name of the variable be?
const MAX_NUM_THREADS_ENV_VARIABLE: &str = "ORX_PARALLEL_MAX_NUM_THREADS";

pub fn maximum_num_threads(input_len: Option<usize>, num_threads: NumThreads) -> usize {
    let max_num_threads = max_num_threads_by_env_variable().unwrap_or(usize::MAX);

    match num_threads {
        NumThreads::Auto => from_auto_num_threads(input_len),
        NumThreads::Max(x) => from_max_num_threads(input_len, x.into()),
    }
    .max(1)
    .min(max_num_threads)
}

fn from_auto_num_threads(input_len: Option<usize>) -> usize {
    match std::thread::available_parallelism() {
        Err(e) => {
            debug_assert!(
                false,
                "Failed to get maximum available parallelism (std::thread::available_parallelism()): {e}",
            );
            input_len
                .unwrap_or(MAX_UNSET_NUM_THREADS)
                .min(MAX_UNSET_NUM_THREADS)
        }
        Ok(available_threads) => input_len
            .unwrap_or(MAX_UNSET_NUM_THREADS)
            .min(available_threads.into()),
    }
}

fn from_max_num_threads(input_len: Option<usize>, max_num_threads: usize) -> usize {
    // TODO: need to get the number of free threads?
    match std::thread::available_parallelism() {
        Err(e) => {
            debug_assert!(
                false,
                "Failed to get maximum available parallelism (std::thread::available_parallelism()); falling back to sequential execution.: {e}",
            );
            input_len.unwrap_or(max_num_threads).min(max_num_threads)
        }
        Ok(available_threads) => input_len
            .unwrap_or(usize::MAX)
            .min(max_num_threads)
            .min(available_threads.into()),
    }
}

fn max_num_threads_by_env_variable() -> Option<usize> {
    match std::env::var(MAX_NUM_THREADS_ENV_VARIABLE) {
        Ok(s) => {
            match s.parse::<usize>() {
                Ok(0) => {
                    // B: To be consistent with `.par().num_threads(0)` means Auto that can use all threads
                    None
                }
                Ok(x) => Some(x),
                Err(_e) => {
                    // C: How should we handle this error?
                    None
                }
            }
        }
        Err(_e) => {
            // D: Environment variable is not set; we assume there is no limit
            None
        }
    }
}
