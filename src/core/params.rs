use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    pub num_threads: Option<NonZeroUsize>,
    pub chunk_size: Option<NonZeroUsize>,
}

impl Params {
    pub fn run_params(self, input_len: Option<usize>) -> RunParams {
        // TODO!
        RunParams {
            num_threads: self.num_threads.map(|x| x.into()).unwrap_or(4),
            chunk_size: self.chunk_size.map(|x| x.into()).unwrap_or(1),
        }
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
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RunParams {
    pub num_threads: usize,
    pub chunk_size: usize,
}
