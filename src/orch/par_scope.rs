use super::par_handle::ParHandle;

pub trait ParScope<'env, 'scope>
where
    'env: 'scope,
{
    type Handle<T>: ParHandle<'scope, T>
    where
        Self: 'scope,
        T: 'scope;

    fn spawn<F, T>(&'scope self, f: F) -> Self::Handle<T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope;
}
