use super::run_params::RunParams;
use std::num::NonZeroUsize;

const MAX_UNSET_NUM_THREADS: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    pub num_threads: Option<NonZeroUsize>,
    pub chunk_size: Option<NonZeroUsize>,
}

impl Params {
    pub fn run_params(self, input_len: Option<usize>) -> RunParams {
        let available_threads = std::thread::available_parallelism();

        let num_threads = self
            .num_threads
            .map(|x| {
                let n: usize = x.into();
                match &available_threads {
                    Err(e) => {
                        debug_assert!(false, "Failed to get maximum available parallelism (std::thread::available_parallelism()); falling back to sequential execution.: {}", e);
                        n
                    },
                    Ok(available) => n.min(available.clone().into()),
                }
            })
            .unwrap_or_else(|| Self::auto_num_threads(input_len, available_threads));

        let chunk_size = self
            .chunk_size
            .map(NonZeroUsize::into)
            .unwrap_or_else(|| Self::auto_chunk_size(input_len, num_threads));

        RunParams::new(num_threads, chunk_size)
    }

    pub fn with_num_threads(self, num_threads: usize) -> Self {
        Self {
            num_threads: Some(
                NonZeroUsize::new(num_threads).expect("Number of threads must be positive"),
            ),
            chunk_size: self.chunk_size,
        }
    }

    pub fn with_chunk_size(self, chunk_size: usize) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: Some(NonZeroUsize::new(chunk_size).expect("Chunk size must be positive")),
        }
    }

    fn auto_num_threads(
        input_len: Option<usize>,
        available_threads: Result<NonZeroUsize, std::io::Error>,
    ) -> usize {
        match available_threads {
            Err(e) => {
                debug_assert!(false, "Failed to get maximum available parallelism (std::thread::available_parallelism()); falling back to sequential execution.: {}", e);
                input_len
                    .unwrap_or(MAX_UNSET_NUM_THREADS)
                    .min(MAX_UNSET_NUM_THREADS)
            }
            Ok(available_threads) => input_len
                .unwrap_or(MAX_UNSET_NUM_THREADS)
                .min(available_threads.into()),
        }
    }

    fn auto_chunk_size(input_len: Option<usize>, num_threads: usize) -> usize {
        fn find_chunk_size(len: usize, num_threads: usize) -> usize {
            let mut chunk_size = 1 << 14;

            loop {
                let one_round_len = chunk_size * num_threads;

                // heterogeneity buffer
                let min_required_len = one_round_len * 3;
                if len >= min_required_len {
                    break;
                }

                // len is still greater, we don't want chunk size to be too small
                if len >= one_round_len && chunk_size <= 32 {
                    break;
                }

                // absolute breaking condition
                if chunk_size == 1 {
                    break;
                }

                chunk_size = chunk_size >> 1;
            }

            chunk_size
        }

        match input_len {
            None => 1,
            Some(len) => find_chunk_size(len, num_threads),
        }
    }
}
