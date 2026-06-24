//! Example demonstrating compile-time-known task queues executed in parallel.
//!
//! The Task trait enables compile-time known task types, while `TaskSingle` and `TaskQueue`
//! provide a typed queue that can run inline or through a thread pool.
//!
//! To run: cargo run --example scope_api --features std

use orx_parallel::Pool;
use orx_parallel::scope::{Task, TaskQueueTrait, TaskSingle};
use std::time::{Duration, Instant};

// ============================================================================
// Compile-Time Known Task Queue
// ============================================================================

struct DelayedDoubleTask {
    value: i32,
    delay_ms: u64,
}

impl Task for DelayedDoubleTask {
    type Output = i32;

    fn run(self) -> Self::Output {
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        println!("  Running DelayedDoubleTask with value {}", self.value);
        self.value * 2
    }
}

struct SquareTask {
    value: i32,
}

impl Task for SquareTask {
    type Output = i32;

    fn run(self) -> Self::Output {
        println!("  Running SquareTask with value {}", self.value);
        self.value * self.value
    }
}

fn main() {
    println!("Scope Module - Parallel Task Queue\n");
    println!("==================================\n");

    let inline_queue = TaskSingle::new(DelayedDoubleTask {
        value: 21,
        delay_ms: 30,
    })
    .push_back(SquareTask { value: 7 });

    let inline_start = Instant::now();
    let inline_result = inline_queue.run_inline();
    let inline_elapsed = inline_start.elapsed();

    println!("Inline result: {:?}", inline_result);
    println!("Inline elapsed: {:?}\n", inline_elapsed);
    assert_eq!(inline_result, (42, 49));

    let parallel_queue = TaskSingle::new(DelayedDoubleTask {
        value: 21,
        delay_ms: 30,
    })
    .push_back(SquareTask { value: 7 });

    let mut pool = Pool::once(2);
    let parallel_start = Instant::now();
    let parallel_result = parallel_queue.run_with_pool(&mut pool);
    let parallel_elapsed = parallel_start.elapsed();

    println!("Parallel result: {:?}", parallel_result);
    println!("Parallel elapsed: {:?}\n", parallel_elapsed);
    assert_eq!(parallel_result, (42, 49));

    println!("✓ Parallel task queue execution complete");
    println!("\nKey Benefits:");
    println!("  - No heap allocation for task storage");
    println!("  - No dynamic dispatch in the queue structure");
    println!("  - Full type safety for heterogeneous tasks");
    println!("  - The same queue can run inline or via a pool");
}
