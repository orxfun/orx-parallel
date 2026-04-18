use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;
use orx_iterable::Iterable;
use std::{println, time::Instant};

pub struct RunnerWithDiagnostics<R: ParRunner>(R);

impl<R: ParRunner> RunnerWithDiagnostics<R> {
    pub fn new(inner: R) -> Self {
        Self(inner)
    }
}

impl<R: ParRunner> ParRunner for RunnerWithDiagnostics<R> {
    type Pool = R::Pool;

    type State = StateWithDiagnostics<R::State>;

    type ChunkState = ChunkStateWithDiagnostics<R::ChunkState>;

    fn pool(&self) -> &Self::Pool {
        self.0.pool()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        self.0.pool_mut()
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        R::do_spawn_new(spawned, &state.inner)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let inner = self.0.new_state(params, max_num_threads, initial_len);
        StateWithDiagnostics::new(max_num_threads, inner)
    }

    fn begin_thread(state: &Self::State, th_idx: usize) {
        R::begin_thread(&state.inner, th_idx);
    }

    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize {
        R::next_chunk_size(&state.inner, remaining)
    }

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState {
        let inner = R::begin_chunk(th_idx, chunk_size);
        ChunkStateWithDiagnostics {
            inner,
            th_idx,
            chunk_size,
        }
    }

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        state
            .task_counts
            .push(chunk_state.th_idx, chunk_state.chunk_size);
        R::complete_chunk(&state.inner, chunk_state.inner);
    }

    fn complete_thread(state: &Self::State, th_idx: usize) {
        R::complete_thread(&state.inner, th_idx);
    }

    fn complete_computation(state: Self::State) {
        R::complete_computation(state.inner);
        state.task_counts.display();
    }
}

struct TaskCounts(Vec<Vec<usize>>);

impl TaskCounts {
    fn new(max_num_threads: usize) -> Self {
        Self((0..max_num_threads).map(|_| Vec::new()).collect())
    }

    fn push(&self, th_idx: usize, chunk_size: usize) {
        let vec = unsafe { &mut *(self.0.as_ptr().add(th_idx) as *mut Vec<usize>) };
        vec.push(chunk_size);
    }

    fn display(self) {
        let counts: Vec<_> = self.0;
        let used_threads = counts
            .iter()
            .filter(|x| x.iter().sum::<usize>() > 0)
            .count();

        println!("\n# Parallel Executor Diagnostics");
        println!("- Number of available threads = {}", counts.len());
        println!("- Number of actually used threads = {}", used_threads);

        println!("- [Thread idx]: num_calls, num_tasks, avg_chunk_size, first_chunk_sizes");

        for (thread_idx, task_counts) in counts.iter().enumerate() {
            let total: usize = task_counts.iter().sum();
            if total > 0 {
                let num_calls = task_counts.len();
                let avg_chunk_size = match num_calls {
                    0 => 0,
                    n => total / n,
                };
                let first_chunks: Vec<_> = task_counts.iter().copied().take(10).collect();

                println!(
                    "  - [{thread_idx}]: {num_calls}, {total}, {avg_chunk_size},\t{first_chunks:?}",
                );
            }
        }
    }
}

struct ThreadLifetimes {
    begin: Vec<Option<Instant>>,
    end: Vec<Option<Instant>>,
}

impl ThreadLifetimes {
    fn new(max_num_threads: usize) -> Self {
        Self {
            begin: (0..max_num_threads).map(|_| None).collect(),
            end: (0..max_num_threads).map(|_| None).collect(),
        }
    }

    fn begin(&mut self, th_idx: usize) {
        self.begin[th_idx] = Some(Instant::now());
    }

    fn end(&mut self, th_idx: usize) {
        self.end[th_idx] = Some(Instant::now());
    }
}

pub struct StateWithDiagnostics<S> {
    inner: S,
    task_counts: TaskCounts,
    thread_lifetimes: ThreadLifetimes,
}

impl<S> StateWithDiagnostics<S> {
    pub fn new(max_num_threads: usize, inner: S) -> Self {
        Self {
            inner,
            task_counts: TaskCounts::new(max_num_threads),
            thread_lifetimes: ThreadLifetimes::new(max_num_threads),
        }
    }
}

pub struct ChunkStateWithDiagnostics<C> {
    inner: C,
    th_idx: usize,
    chunk_size: usize,
}
