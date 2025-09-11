use crate::orch::{ParHandle, ParScope, thread_pool::par_handle::JoinResult};

impl<'scope, T> ParHandle<'scope, T> for std::thread::ScopedJoinHandle<'scope, T> {
    fn join(self) -> JoinResult<T> {
        std::thread::ScopedJoinHandle::join(self)
    }

    fn is_finished(&self) -> bool {
        std::thread::ScopedJoinHandle::is_finished(self)
    }
}

impl<'scope, 'env> ParScope<'scope, 'env> for std::thread::Scope<'scope, 'env> {
    type Handle<T>
        = std::thread::ScopedJoinHandle<'scope, T>
    where
        Self: 'scope,
        T: 'scope;

    fn spawn<F, T>(&'scope self, f: F) -> Self::Handle<T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        self.spawn(f)
    }
}
