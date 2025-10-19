use orx_concurrent_bag::ConcurrentBag;

pub struct SharedStateWithDiagnostics<S> {
    inner: S,
    task_counts: ConcurrentBag<(usize, Vec<usize>)>, // (thread_idx, chunk sizes)
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

    pub fn add_task_counts_of_thread(&self, thread_idx: usize, chunk_sizes: Vec<usize>) {
        self.task_counts.push((thread_idx, chunk_sizes));
    }
}
