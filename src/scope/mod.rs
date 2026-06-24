//! Parallel scope API for structured concurrency and task composition.
//!
//! This module provides two complementary ways to execute parallel work:
//!
//! ## 1. Zero-Cost Static Task Types (Compile-Time Known)
//!
//! For workloads where task types are known at compile time, implement the `Task` trait,
//! build a queue with `TaskSingle<T>::push_back(...)`, and execute it either inline or
//! through a pool with `run_with_pool(...)`.
//!
//! ```ignore
//! use orx_parallel::scope::{Task, TaskSingle};
//!
//! struct MyTask;
//! impl Task for MyTask {
//!     type Output = i32;
//!     fn run(self) -> Self::Output { 42 }
//! }
//!
//! use orx_parallel::Pool;
//!
//! let queue = TaskSingle::new(MyTask).push_back(MyTask);
//!
//! let inline_result = queue.run_inline();
//!
//! let queue = TaskSingle::new(MyTask).push_back(MyTask);
//! let mut pool = Pool::once(2);
//! let parallel_result = queue.run_with_pool(&mut pool);
//! ```
//!
//! This approach is ideal for:
//! - Fixed-structure task DAGs
//! - Performance-critical sections where dispatch overhead matters
//! - Type-safe, heterogeneous task composition
//!
//! ## 2. Dynamic Task Groups (Runtime Workload)
//!
//! For cases where tasks are supplied at runtime (e.g., plugins, user-defined work),
//! use a `DynamicGroup`.
//!
//! ```ignore
//! use orx_parallel::scope::DynamicGroup;
//! use orx_parallel::Pool;
//!
//! let mut group = DynamicGroup::new();
//! for user_task in user_supplied_tasks {
//!     group.spawn(user_task);
//! }
//!
//! let mut pool = Pool::once(8);
//! pool.scoped_computation(|scope| {
//!     group.execute(scope);
//! });
//! ```
//!
//! This approach is ideal for:
//! - User-provided or plugin-based work
//! - Variable-sized workloads
//! - Simple task spawning and joining

mod handle;
mod task;

pub use handle::DynamicGroup;
pub use task::{Task, TaskQueue, TaskQueueTrait, TaskSingle};

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleTask(i32);
    impl Task for SimpleTask {
        type Output = i32;
        fn run(self) -> Self::Output {
            self.0 * 2
        }
    }

    #[test]
    fn test_task_trait() {
        let task = SimpleTask(21);
        let result = task.run();
        assert_eq!(result, 42);
    }
}
