pub struct SharedStateWithDiagnostics<S> {
    inner: S,
}

impl<S> SharedStateWithDiagnostics<S> {
    #[inline(always)]
    pub fn inner(&self) -> &S {
        &self.inner
    }
}
