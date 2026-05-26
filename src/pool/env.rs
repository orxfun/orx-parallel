#[cfg(feature = "std")]
const MAX_NUM_THREADS_ENV_VARIABLE: &str = "ORX_PARALLEL_MAX_NUM_THREADS";

#[cfg(feature = "std")]
const MAX_UNSET_NUM_THREADS: core::num::NonZeroUsize = core::num::NonZeroUsize::new(8).expect(">0");

#[cfg(feature = "std")]
pub fn max_num_threads_by_env_variable() -> Option<core::num::NonZeroUsize> {
    match std::env::var(MAX_NUM_THREADS_ENV_VARIABLE) {
        Ok(s) => match s.parse::<usize>() {
            Ok(x) => core::num::NonZeroUsize::new(x), // None if 0; Some(x) if x is set to a positive bound
            Err(_e) => None,                          // not a number, ignored assuming no bound
        },
        Err(_e) => None, // not set, no bound
    }
}

#[cfg(feature = "std")]
pub fn max_num_threads_by_env_and_resource() -> core::num::NonZeroUsize {
    let env_max_num_threads = max_num_threads_by_env_variable();

    let ava_max_num_threads = std::thread::available_parallelism().ok();

    match (env_max_num_threads, ava_max_num_threads) {
        (Some(env), Some(ava)) => match env < ava {
            true => env,
            false => ava,
        },
        (Some(env), None) => env,
        (None, Some(ava)) => ava,
        (None, None) => MAX_UNSET_NUM_THREADS,
    }
}
