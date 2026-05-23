#[cfg(feature = "std")]
const MAX_NUM_THREADS_ENV_VARIABLE: &str = "ORX_PARALLEL_MAX_NUM_THREADS";

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
