use crate::orch::{ParHandle, ParScope, ParThreadPool, thread_pool::par_handle::JoinResult};
use orx_concurrent_bag::ConcurrentBag;
use scoped_threadpool::{Pool, Scope};

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

pub struct ScopedThreadPoolScope<'scope, 'env>
where
    'env: 'scope,
{
    inner_scope: Scope<'env, 'scope>,
    bag: ConcurrentBag<bool>,
}

impl<'scope, 'env> ParScope<'scope, 'env> for ScopedThreadPoolScope<'scope, 'env>
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

// impl ParThreadPool for Pool {
//     type Scope<'scope, 'env>
//         = ScopedThreadPoolScope<'scope, 'env>
//     where
//         'env: 'scope;

//     fn scope<'env, F, T>(&'env self, f: F) -> T
//     where
//         F: for<'scope> FnOnce(&'scope ScopedThreadPoolScope<'scope, 'env>) -> T,
//     {
//         self.scoped(|s| {});
//         todo!()
//     }
// }

fn main() {
    // Create a threadpool holding 4 threads
    let mut pool = Pool::new(4);

    let mut vec = vec![0, 1, 2, 3, 4, 5, 6, 7];

    // Use the threads as scoped threads that can
    // reference anything outside this closure
    pool.scoped(|scoped| {
        // Create references to each element in the vector ...
        for e in &mut vec {
            // ... and add 1 to it in a seperate thread
            scoped.execute(move || {
                *e += 1;
            });
        }
    });

    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}
