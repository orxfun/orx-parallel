use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;
use std::println;

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

    fn complete_computation(state: Self::State) {
        core::panic!("abc");
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
        let th_counts = unsafe { &mut *(self.0.as_ptr().add(th_idx) as *mut Vec<usize>) };
        th_counts.push(chunk_size);
    }

    fn display(self) {
        let max_th_idx = self
            .0
            .iter()
            .enumerate()
            .filter(|x| x.1.iter().sum::<usize>() > 0)
            .map(|x| x.0)
            .max()
            .unwrap_or(0);

        println!("\n# Parallel Executor Diagnostics");
        println!("\n- Number of threads used = {}", max_th_idx);

        println!("\n- [Thread idx]: num_calls, num_tasks, avg_chunk_size, first_chunk_sizes");

        for (thread_idx, task_counts) in self.0.iter().take(max_th_idx).enumerate() {
            let total: usize = task_counts.iter().sum();
            let num_calls = task_counts.len();
            let avg_chunk_size = match num_calls {
                0 => 0,
                n => total / n,
            };
            let first_chunks: Vec<_> = task_counts.iter().copied().take(10).collect();

            println!(
                "  - [{thread_idx}]: {num_calls}, {total}, {avg_chunk_size}, {first_chunks:?}",
            );
        }
    }
}

pub struct StateWithDiagnostics<S> {
    inner: S,
    task_counts: TaskCounts,
}

impl<S> StateWithDiagnostics<S> {
    pub fn new(max_num_threads: usize, inner: S) -> Self {
        let task_counts = TaskCounts::new(max_num_threads);
        Self { inner, task_counts }
    }
}

pub struct ChunkStateWithDiagnostics<C> {
    inner: C,
    th_idx: usize,
    chunk_size: usize,
}
