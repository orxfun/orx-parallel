use super::{
    computation_kind::ComputationKind,
    parallel_task::{ParallelTask, ParallelTaskWithIdx},
    thread_runner::ThreadRunner,
};
use crate::{computations::Values, parameters::Params};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub trait ParallelRunner: Sized + Sync {
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    fn new(kind: ComputationKind, params: Params, initial_input_len: Option<usize>) -> Self;

    fn new_shared_state(&self) -> Self::SharedState;

    fn do_spawn_new<I>(
        &self,
        num_spawned: usize,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        I: ConcurrentIter;

    fn new_thread_runner(&self, shared_state: &Self::SharedState) -> Self::ThreadRunner;

    fn new_run<I, T>(&self, iter: &I, task: T) -> usize
    where
        I: ConcurrentIter,
        T: ParallelTask<Item = I::Item> + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(|| {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.new_run(iter, shared_state, &task);
                });
            }
        });
        num_spawned
    }

    fn new_run_idx<I, T>(&self, iter: &I, task: T) -> usize
    where
        I: ConcurrentIter,
        T: ParallelTaskWithIdx<Item = I::Item> + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(|| {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.new_run_with_idx(iter, shared_state, &task);
                });
            }
        });
        num_spawned
    }

    // zzzzzzzzzzzz

    fn run<I, T>(&self, iter: &I, transform: &T) -> usize
    where
        I: ConcurrentIter,
        T: Fn(I::Item) + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.run(iter, shared_state, transform);
                });
            }
        });
        num_spawned
    }

    fn run_with_idx<I, T>(&self, iter: &I, transform: &T) -> usize
    where
        I: ConcurrentIter,
        T: Fn((usize, I::Item)) + Sync,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.run_with_idx(iter, shared_state, transform);
                });
            }
        });
        num_spawned
    }

    // ZZZZZZZZZZZZZZZ

    fn mfm_collect_with_idx<I, T, O, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Vec<Vec<(usize, O)>>)
    where
        I: ConcurrentIter,
        M1: Fn(I::Item) -> T + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> O + Send + Sync,
        O: Send,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        let vectors = std::thread::scope(|s| {
            let mut handles = vec![];

            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                handles.push(s.spawn(move || {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.mfm_collect_with_idx(iter, shared_state, map1, filter, map2)
                }));
            }

            let mut vectors = Vec::with_capacity(handles.len());
            for x in handles {
                vectors.push(x.join().expect("failed to join the thread"));
            }
            vectors
        });

        (num_spawned, vectors)
    }

    fn mfm_collect_to_vecs<I, T, Vt, O, Vo, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Vec<Vec<(usize, O)>>)
    where
        I: ConcurrentIter,
        Vt: Values<Item = T>,
        O: Send + Sync,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        O: Send,
    {
        let state = self.new_shared_state();
        let shared_state = &state;

        let mut num_spawned = 0;
        let vectors = std::thread::scope(|s| {
            let mut handles = vec![];

            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                handles.push(s.spawn(move || {
                    let thread_runner = self.new_thread_runner(shared_state);
                    thread_runner.mfm_collect_heap(iter, shared_state, map1, filter, map2)
                }));
            }

            let mut vectors = Vec::with_capacity(handles.len());
            for x in handles {
                vectors.push(x.join().expect("failed to join the thread"));
            }
            vectors
        });

        (num_spawned, vectors)
    }
}
