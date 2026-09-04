/// A scope within which work can be spawned onto a [`ThreadPool`], bounded by the
/// lifetimes of the borrowed environment.
///
/// [`ThreadPool`]: crate::ThreadPool
pub trait Scope<'s, 'env, 'scope> {
    /// Runs `work` within this scope, either immediately or on a worker thread.
    fn run<W>(&self, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;
}
