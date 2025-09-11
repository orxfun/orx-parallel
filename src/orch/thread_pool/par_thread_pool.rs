use super::par_scope::ParScope;

pub trait ParThreadPool {
    type Scope<'scope, 'env>: ParScope<'scope, 'env>
    where
        'env: 'scope;

    fn scope<'env, F, T>(&'env self, f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope Self::Scope<'scope, 'env>) -> T;
}
