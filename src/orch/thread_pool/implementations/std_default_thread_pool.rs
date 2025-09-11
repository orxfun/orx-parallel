use crate::orch::{ParHandle, ParScope, ParThreadPool, thread_pool::par_handle::JoinResult};

pub struct StdHandle<'scope, T>(std::thread::ScopedJoinHandle<'scope, T>);

impl<'scope, T> ParHandle<'scope, T> for StdHandle<'scope, T> {
    fn join(self) -> JoinResult<T> {
        self.0.join()
    }

    fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}

impl<'scope, 'env> ParScope<'scope, 'env> for std::thread::Scope<'scope, 'env> {
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

#[derive(Default)]
pub struct StdDefaultThreadPool;

impl ParThreadPool for StdDefaultThreadPool {
    type Scope<'scope, 'env>
        = std::thread::Scope<'scope, 'env>
    where
        'env: 'scope;

    fn scope<'env, F, T>(&'env self, f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope std::thread::Scope<'scope, 'env>) -> T,
    {
        std::thread::scope(f)
    }
}
