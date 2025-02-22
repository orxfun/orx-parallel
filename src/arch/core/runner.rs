use crate::{
    chunk_size::ResolvedChunkSize,
    core::runner_settings::{chunk_size, num_threads},
    Params,
};
use orx_concurrent_iter::{ConcurrentIterX, HasMore};
use std::hint::black_box;

const LAG_PERIODICITY: usize = 4;

#[derive(Clone, Copy, Debug)]
pub enum ParTask {
    Collect,
    EarlyReturn,
    Reduce,
}

#[derive(Clone, Copy, Debug)]
pub struct Runner {
    _task: ParTask,
    input_len: Option<usize>,
    max_num_threads: usize,
    chunk_size: ResolvedChunkSize,
}

impl Runner {
    fn new(params: Params, task: ParTask, input_len: Option<usize>) -> Self {
        let max_num_threads = num_threads::calc_num_threads(input_len, params.num_threads).max(1);
        let chunk_size =
            chunk_size::calc_chunk_size(task, input_len, max_num_threads, params.chunk_size);

        Self {
            _task: task,
            input_len,
            max_num_threads,
            chunk_size,
        }
    }

    pub fn do_spawn(&self, num_spawned: usize, has_more: HasMore) -> bool {
        match num_spawned {
            x if x >= self.max_num_threads - 1 => false,
            _ => !matches!(has_more, HasMore::No),
        }
    }

    pub fn next_chunk_size(&self, num_spawned: usize, has_more: HasMore) -> Option<usize> {
        match has_more {
            HasMore::No => None,
            HasMore::Maybe => self.next_chunk_size_unknown_len(num_spawned),
            HasMore::Yes(remaining_len) => {
                self.next_chunk_size_known_len(num_spawned, remaining_len)
            }
        }
    }

    fn next_chunk_size_unknown_len(&self, num_spawned_threads: usize) -> Option<usize> {
        match num_spawned_threads {
            x if x >= self.max_num_threads - 1 => None,
            _ => Some(self.chunk_size.inner()),
        }
    }

    fn next_chunk_size_known_len(
        &self,
        num_spawned_threads: usize,
        remaining_len: usize,
    ) -> Option<usize> {
        match num_spawned_threads {
            x if x >= self.max_num_threads - 1 => None,
            _ => match self.chunk_size {
                ResolvedChunkSize::Exact(x) => Some(x),
                ResolvedChunkSize::Min(x) => {
                    let chunk_size = match num_spawned_threads {
                        0 => x,
                        _ => {
                            // x
                            let len = self.input_len.unwrap_or(usize::MAX);
                            let done = len - remaining_len;

                            let done_per_thread = done / num_spawned_threads;
                            let num_chunks_per_thread = (done_per_thread / x).max(1);
                            let num_chunks_per_thread = num_chunks_per_thread.max(1);
                            num_chunks_per_thread * x
                        }
                    };

                    Some(chunk_size)
                }
            },
        }
    }

    pub fn run<I, F>(params: Params, task_type: ParTask, iter: &I, thread_task: &F) -> usize
    where
        I: ConcurrentIterX,
        F: Fn(usize) + Sync,
    {
        let runner = Self::new(params, task_type, iter.try_get_len());

        let mut num_spawned = 0;

        std::thread::scope(|s| {
            let mut chunk: usize = runner.chunk_size.inner();
            'lag_period: loop {
                for _ in 0..LAG_PERIODICITY {
                    match runner.do_spawn(num_spawned, iter.has_more()) {
                        false => break 'lag_period,
                        true => {
                            s.spawn(move || thread_task(chunk));
                            num_spawned += 1;
                        }
                    }
                }

                lag();
                match runner.next_chunk_size(num_spawned, iter.has_more()) {
                    None => break 'lag_period,
                    Some(c) => chunk = c,
                }
            }

            s.spawn(move || thread_task(chunk));
            num_spawned += 1;
        });

        num_spawned
    }

    pub fn run_map<I, F, Out>(
        params: Params,
        task_type: ParTask,
        iter: &I,
        thread_task: &F,
    ) -> Vec<Out>
    where
        I: ConcurrentIterX,
        F: Fn(usize) -> Out + Sync,
        Out: Send + Sync,
    {
        let runner = Self::new(params, task_type, iter.try_get_len());

        let mut num_spawned = 0;

        std::thread::scope(|s| {
            let mut handles = vec![];
            let mut chunk: usize = runner.chunk_size.inner();
            'lag_period: loop {
                for _ in 0..LAG_PERIODICITY {
                    match runner.do_spawn(num_spawned, iter.has_more()) {
                        false => break 'lag_period,
                        true => {
                            handles.push(s.spawn(move || thread_task(chunk)));
                            num_spawned += 1;
                        }
                    }
                }

                lag();
                match runner.next_chunk_size(num_spawned, iter.has_more()) {
                    None => break 'lag_period,
                    Some(c) => chunk = c,
                }
            }

            handles.push(s.spawn(move || thread_task(chunk)));
            num_spawned += 1;

            let mut vec = vec![];
            for x in handles {
                vec.push(x.join().expect("failed to join the thread"));
            }
            vec
        })
    }

    pub fn reduce<I, F, T, R>(
        params: Params,
        task_type: ParTask,
        iter: &I,
        thread_task: &F,
        reduce: R,
    ) -> (usize, Option<T>)
    where
        I: ConcurrentIterX,
        F: Fn(usize) -> T + Sync,
        T: Send,
        R: Fn(T, T) -> T,
    {
        let runner = Self::new(params, task_type, iter.try_get_len());

        std::thread::scope(|s| {
            let mut threads = Vec::with_capacity(runner.max_num_threads);

            let mut chunk: usize = runner.chunk_size.inner();
            'lag_period: loop {
                for _ in 0..LAG_PERIODICITY {
                    match runner.do_spawn(threads.len(), iter.has_more()) {
                        false => break 'lag_period,
                        true => threads.push(s.spawn(move || thread_task(chunk))),
                    }
                }

                lag();
                match runner.next_chunk_size(threads.len(), iter.has_more()) {
                    None => break 'lag_period,
                    Some(c) => chunk = c,
                }
            }

            threads.push(s.spawn(move || thread_task(chunk)));

            let num_threads = threads.len();
            let result = threads
                .into_iter()
                .map(|x| x.join().expect("Failed to join thread"))
                .reduce(reduce);

            (num_threads, result)
        })
    }
}

fn lag() {
    fn fibonacci(n: i32) -> i32 {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            let c = i32::saturating_add(a, b);
            a = b;
            b = c;
        }
        a
    }

    assert!(black_box(fibonacci(1 << 16)) > 0);
}
