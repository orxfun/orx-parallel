use crate::orch::ParThreadPool;

#[derive(Default)]
pub struct StdOsThreadPool;

impl ParThreadPool for StdOsThreadPool {
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
