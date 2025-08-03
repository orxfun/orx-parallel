use orx_parallel::*;
use std::cell::UnsafeCell;

const N: u64 = 10_000_000;
const MAX_NUM_THREADS: usize = 8;

// just some work
fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

#[derive(Default, Debug)]
struct ThreadMetrics {
    num_items_handled: usize,
}

struct ThreadMetricsWriter<'a> {
    metrics_ref: &'a mut ThreadMetrics,
}
impl<'a> ThreadMetricsWriter<'a> {
    fn increment_num_items_handled(&mut self) {
        self.metrics_ref.num_items_handled += 1;
    }
}

#[derive(Default)]
struct ComputationMetrics {
    thread_metrics: UnsafeCell<[ThreadMetrics; MAX_NUM_THREADS]>,
}

impl ComputationMetrics {
    unsafe fn create<'a>(&mut self, thread_idx: usize) -> ThreadMetricsWriter<'a> {
        // SAFETY: here we create a mutable variable to the thread_idx-th metrics
        // * If we call this method multiple times with the same index,
        //   we create multiple mutable references to the same ThreadMetrics,
        //   which would lead to a race condition.
        // * We must make sure that `create` is called only once per thread.
        // * If we use `create` within the `using` call to create mutable values
        //   used by the threads, we are certain that the parallel computation
        //   will only call this method once per thread; hence, it will not
        //   cause the race condition.
        // * On the other hand, we must ensure that we do not call this method
        //   externally.
        let array = unsafe { &mut *self.thread_metrics.get() };
        let writer = ThreadMetricsWriter {
            metrics_ref: &mut array[thread_idx],
        };

        writer
    }
}

fn main() {
    let mut metrics = ComputationMetrics::default();

    let input: Vec<u64> = (0..N).collect();

    let sum = input
        .par()
        // SAFETY: we do not call `create` externally; it is safe if it is called
        // only by the parallel computation.
        .using(|t| unsafe { metrics.create(t) })
        .map(|m: &mut ThreadMetricsWriter<'_>, i| {
            // collect some useful metrics
            m.increment_num_items_handled();

            // actual work
            fibonacci((*i % 50) + 1) % 100
        })
        .num_threads(MAX_NUM_THREADS)
        .sum();

    println!("\nINPUT-LEN = {N}");
    println!("SUM = {sum}");

    println!("\n\n");

    println!("COLLECTED METRICS PER THREAD");
    for (thread_idx, metrics) in metrics.thread_metrics.get_mut().iter().enumerate() {
        println!("* thread {thread_idx}: {metrics:?}");
    }
    let total_by_metrics: usize = metrics
        .thread_metrics
        .get_mut()
        .iter()
        .map(|x| x.num_items_handled)
        .sum();
    println!("\n-> total num_items_handled by collected metrics: {total_by_metrics:?}\n");

    assert_eq!(N as usize, total_by_metrics);
}
