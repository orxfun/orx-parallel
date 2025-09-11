use super::super::par_handle::{JoinResult, ParHandle};
use super::super::par_scope::ParScope;
use crate::{ParallelRunner, orch::Orchestrator};
use std::marker::PhantomData;

pub struct StdHandle<'scope, T>(std::thread::ScopedJoinHandle<'scope, T>);

impl<'scope, T> ParHandle<'scope, T> for StdHandle<'scope, T> {
    fn join(self) -> JoinResult<T> {
        self.0.join()
    }
}

impl<'env, 'scope> ParScope<'env, 'scope> for std::thread::Scope<'scope, 'env> {
    type Handle<T>
        = StdHandle<'scope, T>
    where
        Self: 'scope,
        T: 'scope;

    fn spawn<F, T>(&'scope self, f: F) -> Self::Handle<T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        StdHandle(self.spawn(f))
    }
}

pub struct StdOrchestrator<R>
where
    R: ParallelRunner,
{
    r: PhantomData<R>,
}

impl<R> Default for StdOrchestrator<R>
where
    R: ParallelRunner,
{
    fn default() -> Self {
        Self { r: PhantomData }
    }
}

impl<R> Orchestrator for StdOrchestrator<R>
where
    R: ParallelRunner,
{
    type Runner = R;

    type Scope<'env, 'scope>
        = std::thread::Scope<'scope, 'env>
    where
        'env: 'scope;

    fn scope<'env, F, T>(f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope std::thread::Scope<'scope, 'env>) -> T,
    {
        std::thread::scope(|s| f(s))
    }
}
