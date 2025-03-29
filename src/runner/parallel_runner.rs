use super::{
    computation_kind::ComputationKind,
    parallel_task::{ParallelTask, ParallelTaskWithIdx},
    thread_runner::ThreadRunner,
};
use crate::{computations::Values, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

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

    fn run<I, T>(&self, iter: &I, task: T) -> usize
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
                    thread_runner.run(iter, shared_state, &task);
                });
            }
        });
        num_spawned
    }

    fn run_with_idx<I, T>(&self, iter: &I, task: T) -> usize
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
                    thread_runner.run_with_idx(iter, shared_state, &task);
                });
            }
        });
        num_spawned
    }

    fn x_collect_with_idx<I, Vo, M1>(
        &self,
        iter: &I,
        map1: &M1,
    ) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
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
                    thread_runner.x_collect_with_idx(iter, shared_state, map1)
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

    fn xfx_collect_with_idx<I, Vt, Vo, M1, F, M2>(
        &self,
        iter: &I,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> (usize, Vec<Vec<(usize, Vo::Item)>>)
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
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
                    thread_runner.xfx_collect_with_idx(iter, shared_state, map1, filter, map2)
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
