use orx_concurrent_bag::ConcurrentBag;

pub struct SharedStateWithDiagnostics<S> {
    inner: S,
    task_counts: ConcurrentBag<(usize, usize)>, // (thread_idx, chunk_size)
}

impl<S> SharedStateWithDiagnostics<S> {
    pub fn new(inner: S) -> Self {
        let tasks = ConcurrentBag::new();
        Self {
            inner,
            task_counts: tasks,
        }
    }

    #[inline(always)]
    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn add_task_count(&self, thread_idx: usize, chunk_size: usize) {
        self.task_counts.push((thread_idx, chunk_size));
    }
}
