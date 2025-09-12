use crate::orch::{ParHandle, ParScope, ParThreadPool, thread_pool::par_handle::JoinResult};
use orx_concurrent_bag::ConcurrentBag;
use std::marker::PhantomData;
use threadpool::ThreadPool;

pub struct ThreadPoolHandle<'scope, T> {
    idx: usize,
    result: Option<T>,
    bag: &'scope ConcurrentBag<bool>,
}

impl<'scope, T> ParHandle<'scope, T> for ThreadPoolHandle<'scope, T> {
    fn join(self) -> JoinResult<T> {
        todo!()
    }

    fn is_finished(&self) -> bool {
        todo!()
    }
}

pub struct ThreadPoolScope<'scope, 'env>
where
    'env: 'scope,
{
    pool: &'env ThreadPool,
    bag: ConcurrentBag<bool>,
    p: PhantomData<&'scope ()>,
}

impl<'scope, 'env> ParScope<'scope, 'env> for ThreadPoolScope<'scope, 'env>
where
    'env: 'scope,
{
    type Handle<T>
        = ThreadPoolHandle<'scope, T>
    where
        Self: 'scope,
        T: 'scope;

    fn spawn<F, T>(&'scope self, f: F) -> Self::Handle<T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        todo!()
    }
}

impl ParThreadPool for ThreadPool {
    type Scope<'scope, 'env>
        = ThreadPoolScope<'scope, 'env>
    where
        'env: 'scope;

    fn scope<'env, F, T>(&'env self, f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope ThreadPoolScope<'scope, 'env>) -> T,
    {
        todo!()
    }
}
