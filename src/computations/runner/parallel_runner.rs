use super::thread_runner::ThreadRunner;
use crate::parameters::Params;
use orx_concurrent_iter::{ConcurrentIter, Element, Enumeration};

pub trait ParallelRunner: Sized {
    type SharedState: Send + Sync;

    type ThreadRunner: ThreadRunner<SharedState = Self::SharedState>;

    fn new() -> Self;

    fn new_shared_state() -> Self::SharedState;

    fn do_spawn_new<E, I>(
        &self,
        num_spawned: usize,
        shared_state: &Self::SharedState,
        iter: &I,
    ) -> bool
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn run<E, I, T>(self, params: Params, iter: &I, transform: &T)
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
        T: Fn(<E::Element as Element>::ElemOf<I::Item>) + Sync,
    {
        let state = Self::new_shared_state();
        let shared_state = &state;
        let mut num_spawned = 0;

        std::thread::scope(|s| {
            while self.do_spawn_new(num_spawned, shared_state, iter) {
                num_spawned += 1;
                s.spawn(move || {
                    let thread_runner = Self::ThreadRunner::new(params.chunk_size);
                    thread_runner.run(iter, shared_state, transform);
                });
            }
            // loop {
            // match self.do_spawn_new(num_spawned, &shared_state, iter) {
            //     true => {}
            //     false => break,
            // }
        });
    }
}
