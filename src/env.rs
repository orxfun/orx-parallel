use core::num::NonZeroUsize;

#[cfg(feature = "std")]
const MAX_NUM_THREADS_ENV_VARIABLE: &str = "ORX_PARALLEL_MAX_NUM_THREADS";

pub fn max_num_threads_by_env_variable() -> Option<NonZeroUsize> {
    #[cfg(feature = "std")]
    match std::env::var(MAX_NUM_THREADS_ENV_VARIABLE) {
        Ok(s) => match s.parse::<usize>() {
            Ok(0) => None, // consistent with .num_threads(0) representing no bound
            Ok(x) => Some(NonZeroUsize::new(x).expect("x>0")), // set to a positive bound
            Err(_e) => None, // not a number, ignored assuming no bound
        },
        Err(_e) => None, // not set, no bound
    }

    #[cfg(not(feature = "std"))]
    None
}
