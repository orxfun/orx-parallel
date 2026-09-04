pub trait Scope<'s, 'env, 'scope> {
    fn run<W>(&self, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;
}
