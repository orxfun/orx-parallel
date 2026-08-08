// fn xyz() {
//     use orx_parallel::*;
//     use std::cell::UnsafeCell;

//     const N: u64 = 10_000_000;
//     const MAX_NUM_THREADS: usize = 8;

//     // just some work
//     fn fibonacci(n: u64) -> u64 {
//         let mut a = 0;
//         let mut b = 1;
//         for _ in 0..n {
//             let c = a + b;
//             a = b;
//             b = c;
//         }
//         a
//     }

//     #[derive(Default, Debug)]
//     struct ThreadMetrics {
//         thread_idx: usize,
//         num_items_handled: usize,
//         handled_42: bool,
//         num_filtered_out: usize,
//     }

//     struct ThreadMetricsWriter<'a> {
//         metrics_ref: &'a mut ThreadMetrics,
//     }

//     struct ComputationMetrics {
//         thread_metrics: UnsafeCell<[ThreadMetrics; MAX_NUM_THREADS]>,
//     }
//     impl ComputationMetrics {
//         fn new() -> Self {
//             let mut thread_metrics: [ThreadMetrics; MAX_NUM_THREADS] = Default::default();
//             for i in 0..MAX_NUM_THREADS {
//                 thread_metrics[i].thread_idx = i;
//             }
//             Self {
//                 thread_metrics: UnsafeCell::new(thread_metrics),
//             }
//         }
//     }

//     impl ComputationMetrics {
//         unsafe fn create_for_thread<'a>(&mut self, thread_idx: usize) -> ThreadMetricsWriter<'a> {
//             // SAFETY: here we create a mutable variable to the thread_idx-th metrics
//             // * If we call this method multiple times with the same index,
//             //   we create multiple mutable references to the same ThreadMetrics,
//             //   which would lead to a race condition.
//             // * We must make sure that `create_for_thread` is called only once per thread.
//             // * If we use `create_for_thread` within the `using` call to create mutable values
//             //   used by the threads, we are certain that the parallel computation
//             //   will only call this method once per thread; hence, it will not
//             //   cause the race condition.
//             // * On the other hand, we must ensure that we do not call this method
//             //   externally.
//             let array = unsafe { &mut *self.thread_metrics.get() };
//             ThreadMetricsWriter {
//                 metrics_ref: &mut array[thread_idx],
//             }
//         }
//     }

//     fn main() {
//         let mut metrics = ComputationMetrics::new();

//         let input: Vec<u64> = (0..N).collect();

//         let sum = input
//             .par()
//             // SAFETY: we do not call `create_for_thread` externally;
//             // it is safe if it is called only by the parallel computation.
//             .using(|t| unsafe { metrics.create_for_thread(t) })
//             .map(|m: &mut ThreadMetricsWriter<'_>, i| {
//                 // collect some useful metrics
//                 m.metrics_ref.num_items_handled += 1;
//                 m.metrics_ref.handled_42 |= *i == 42;

//                 // actual work
//                 fibonacci((*i % 50) + 1) % 100
//             })
//             .filter(|m, i| {
//                 let is_even = i % 2 == 0;

//                 if !is_even {
//                     m.metrics_ref.num_filtered_out += 1;
//                 }

//                 is_even
//             })
//             .num_threads(MAX_NUM_THREADS)
//             .sum();

//         println!("\nINPUT-LEN = {N}");
//         println!("SUM = {sum}");

//         println!("\n\n");

//         println!("COLLECTED METRICS PER THREAD");
//         for metrics in metrics.thread_metrics.get_mut().iter() {
//             println!("* {metrics:?}");
//         }
//         let total_by_metrics: usize = metrics
//             .thread_metrics
//             .get_mut()
//             .iter()
//             .map(|x| x.num_items_handled)
//             .sum();
//         println!("\n-> total num_items_handled by collected metrics: {total_by_metrics:?}\n");

//         assert_eq!(N as usize, total_by_metrics);
//     }
// }

#[test]
fn for_doc() {
    use orx_parallel::*;

    #[derive(Default, Debug)]
    struct ThreadMetrics {
        num_items_handled: usize,
        handled_12345: bool,
        num_filtered_out: usize,
    }

    let input: Vec<u64> = (0..1_000_000).collect();

    // define how to create thread-local variables
    let mut thread_metrics = UseVec::new(|_th_idx| ThreadMetrics::default());

    let total = input
        .par()
        .num_threads(8)
        .use_vec(&mut thread_metrics) // ← mutable lend it to parallel iterator
        .map(|metrics, x| {
            metrics.num_items_handled += 1;
            metrics.handled_12345 |= *x == 12345;

            x + x / 7 + 17
        })
        .filter(|metrics, x| match x.is_multiple_of(3) {
            true => true,
            false => {
                metrics.num_filtered_out += 1;
                false
            }
        })
        .sum();
    assert_eq!(total, 190481523804);

    let thread_metrics = thread_metrics.into_vec(); // ← get the created vars back
    for (th_idx, metrics) in thread_metrics.iter().enumerate() {
        println!("[th-{th_idx}]\t{metrics:?}");
    }
}

/*
[th-0]  ThreadMetrics { num_items_handled: 130212, handled_12345: false, num_filtered_out: 86816 }
[th-1]  ThreadMetrics { num_items_handled: 106251, handled_12345: false, num_filtered_out: 70828 }
[th-2]  ThreadMetrics { num_items_handled: 112540, handled_12345: false, num_filtered_out: 75035 }
[th-3]  ThreadMetrics { num_items_handled: 176754, handled_12345: false, num_filtered_out: 117822 }
[th-4]  ThreadMetrics { num_items_handled: 110223, handled_12345: false, num_filtered_out: 73475 }
[th-5]  ThreadMetrics { num_items_handled: 110554, handled_12345: true, num_filtered_out: 73713 }
[th-6]  ThreadMetrics { num_items_handled: 126773, handled_12345: false, num_filtered_out: 84523 }
[th-7]  ThreadMetrics { num_items_handled: 126693, handled_12345: false, num_filtered_out: 84455 }

*/
