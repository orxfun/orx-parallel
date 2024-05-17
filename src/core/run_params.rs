use orx_concurrent_iter::HasMore;
use std::{cmp::Ordering, hint::black_box};

#[derive(Clone, Copy, Debug, Default)]
pub struct RunParams {
    max_num_threads: usize,
    chunk_size: usize,
}

impl RunParams {
    pub fn new(max_num_threads: usize, chunk_size: usize) -> Self {
        assert!(max_num_threads >= 1, "Number of threads must be positive");
        assert!(chunk_size >= 1, "Chunk size must be positive");
        Self {
            max_num_threads,
            chunk_size,
        }
    }

    pub fn is_sequential(&self) -> bool {
        self.max_num_threads < 2
    }

    // pub fn num_threads(&self) -> usize {
    //     self.max_num_threads
    // }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn do_spawn_new(&self, num_spawned_threads: usize, has_more: HasMore) -> bool {
        // let num_remaining = match has_more {
        //     HasMore::Yes(x) => x,
        //     _ => 0,
        // };
        // let str = format!(
        //     "spawned={}  remaining={}  num-chunks={}    condition={}",
        //     num_spawned_threads,
        //     num_remaining,
        //     num_remaining / self.chunk_size,
        //     8 * num_remaining > self.chunk_size
        // );
        // dbg!(&str);

        let result = match self.max_num_threads.cmp(&num_spawned_threads) {
            Ordering::Greater => match has_more {
                HasMore::No => false,
                HasMore::Maybe => true,
                HasMore::Yes(num_remaining) => match num_spawned_threads {
                    0 => true,
                    _ => {
                        let num_chunks = num_remaining / self.chunk_size;
                        match num_chunks {
                            0 => 2 * num_remaining > self.chunk_size,
                            _ => true,
                        }
                    }
                },
            },
            _ => false,
        };

        match (result, num_spawned_threads) {
            (true, 8 | 16) => black_box(fibonacci(1 << 20)) > 0,
            _ => result,
        }
    }
}

fn fibonacci(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}
