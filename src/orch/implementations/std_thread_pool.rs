// use super::super::{
//     par_handle::{JoinResult, ParHandle},
//     par_scope::ParScope,
//     par_thread_pool::ParThreadPool,
// };

// pub struct StdHandle<'scope, T>(std::thread::ScopedJoinHandle<'scope, T>);

// impl<'scope, T> ParHandle<'scope, T> for StdHandle<'scope, T> {
//     fn join(self) -> JoinResult<T> {
//         self.0.join()
//     }

//     fn is_finished(&self) -> bool {
//         self.0.is_finished()
//     }
// }

// impl<'env, 'scope> ParScope<'env, 'scope> for std::thread::Scope<'scope, 'env> {
//     type Handle<T>
//         = StdHandle<'scope, T>
//     where
//         Self: 'scope,
//         T: 'scope;

//     fn spawn<F, T>(&'scope self, f: F) -> Self::Handle<T>
//     where
//         F: FnOnce() -> T + Send + 'scope,
//         T: Send + 'scope,
//     {
//         StdHandle(self.spawn(f))
//     }
// }

// pub struct StdThreadPool;

// impl ParThreadPool for StdThreadPool {
//     type Scope<'env, 'scope>
//         = std::thread::Scope<'scope, 'env>
//     where
//         'env: 'scope;

//     fn scope<'env, F, T>(f: F) -> T
//     where
//         F: for<'scope> FnOnce(&'scope std::thread::Scope<'scope, 'env>) -> T,
//     {
//         std::thread::scope(f)
//     }
// }
