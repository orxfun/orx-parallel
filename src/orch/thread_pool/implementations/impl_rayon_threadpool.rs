use crate::orch::{ParHandle, ParScope, ParThreadPool, thread_pool::par_handle::JoinResult};
use orx_concurrent_bag::ConcurrentBag;
use rayon::{Scope, ThreadPool};

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

impl<'scope, 'env> ParScope<'scope, 'env> for Scope<'scope>
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

// impl ParThreadPool for ThreadPool {
//     type Scope<'scope, 'env>
//         = Scope<'scope>
//     where
//         'env: 'scope;

//     fn scope<'env, F, T>(&'env mut self, f: F) -> T
//     where
//         F: for<'scope> FnOnce(&'scope Scope<'scope>) -> T,
//     {
//         todo!()
//     }
// }
