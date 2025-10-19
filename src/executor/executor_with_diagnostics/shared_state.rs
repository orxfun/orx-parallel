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

    pub fn display(self) {
        let mut task_counts = self.task_counts.into_inner().to_vec();
        task_counts.sort_by_key(|x| x.0);

        println!("\n# Parallel Executor Diagnostics");
        println!("\n- Number of threads used = {}", task_counts.len());
        println!();

        println!("\n- Threads");

        for (thread_idx, task_counts) in task_counts {
            let total: usize = task_counts.iter().sum();
            let num_calls = task_counts.len();
            let avg_chunk_size = match num_calls {
                0 => 0,
                n => total / n,
            };
            let first_chunks: Vec<_> = task_counts.iter().copied().take(10).collect();
            println!("\n- Thread # {}", thread_idx);
            println!("  - total number of calls = {}", num_calls);
            println!("  - total number of tasks = {}", total);
            println!("  - average chunk size = {}", avg_chunk_size);
            println!("  - first chunks sizes = {:?}", first_chunks);
        }
    }
}
