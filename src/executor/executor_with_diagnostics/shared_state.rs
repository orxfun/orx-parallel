use orx_concurrent_bag::ConcurrentBag;

pub struct SharedStateWithDiagnostics<S> {
    inner: S,
    tasks: ConcurrentBag<usize>,
}

impl<S> SharedStateWithDiagnostics<S> {
    pub fn new(inner: S) -> Self {
        let tasks = ConcurrentBag::new();
        Self { inner, tasks }
    }

    #[inline(always)]
    pub fn inner(&self) -> &S {
        &self.inner
    }
}
