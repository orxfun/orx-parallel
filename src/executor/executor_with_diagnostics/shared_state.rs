pub struct SharedStateWithDiagnostics<S> {
    inner: S,
}

impl<S> SharedStateWithDiagnostics<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    #[inline(always)]
    pub fn inner(&self) -> &S {
        &self.inner
    }
}
