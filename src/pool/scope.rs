/// A scope within which work can be spawned onto a [`ThreadPool`], bounded by the
/// lifetimes of the borrowed environment.
///
/// [`ThreadPool`]: crate::ThreadPool
pub trait Scope<'s, 'env, 'scope> {
    /// Runs `work` within this scope on a worker thread of the pool this
    /// scope is created from.
    fn run<W>(&self, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;
}
