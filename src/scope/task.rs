#![allow(missing_docs)]

use crate::ParThreadPool;
use orx_meta::queue::{Queue, QueueSingle, StQueue};
use std::sync::{Arc, Mutex};

/// A unit of work that can be executed and returns a result.
///
/// The `Task` trait is the foundation of zero-cost parallel task composition.
/// Tasks are `Send` to enable execution on thread pools.
///
/// # Example
///
/// ```ignore
/// use orx_parallel::scope::Task;
///
/// struct ProcessData;
///
/// impl Task for ProcessData {
///     type Output = Vec<i32>;
///
///     fn run(self) -> Self::Output {
///         (0..1000).map(|x| x * 2).collect()
///     }
/// }
/// ```
///
/// # Zero-Cost Composition
///
/// The primary benefit of the `Task` trait is enabling heterogeneous collections
/// of tasks with compile-time known types and zero runtime dispatch overhead.
/// This is in contrast to type-erased approaches like `Box<dyn Fn() + Send>`.
pub trait Task: Send {
    /// The type of result produced by executing this task.
    type Output: Send;

    /// Execute the task and return its output.
    fn run(self) -> Self::Output;
}

pub type TaskSingle<T> = QueueSingle<T>;
pub type TaskQueue<F, B> = Queue<F, B>;

pub trait TaskQueuePushBack: StQueue + Sized {
    fn push_back<T: Task>(self, task: T) -> Self::PushBack<T> {
        self.push(task)
    }
}

impl<Q: StQueue> TaskQueuePushBack for Q {}

pub struct TaskJoin<T: Send> {
    result: Arc<Mutex<Option<T>>>,
}

impl<T: Send> Clone for TaskJoin<T> {
    fn clone(&self) -> Self {
        Self {
            result: Arc::clone(&self.result),
        }
    }
}

impl<T: Send> TaskJoin<T> {
    fn new() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
        }
    }

    fn store(&self, value: T) {
        let mut result = self.result.lock().expect("poisoned task result lock");
        *result = Some(value);
    }

    fn join(self) -> T {
        let mut result = self.result.lock().expect("poisoned task result lock");
        result
            .take()
            .expect("task result missing; join must happen after the scope completes")
    }
}

/// A marker trait for task queue implementations.
///
/// A task queue can contain multiple tasks arranged in a hierarchy.
/// This trait enables composing heterogeneous task types while maintaining
/// type safety and avoiding heap allocations for known-at-compile-time task graphs.
pub trait TaskQueueTrait: Send {
    /// The output returned by this queue.
    type Output: Send;

    type JoinHandle: Send;

    /// Execute this queue synchronously on the current thread.
    fn run_inline(self) -> Self::Output;

    fn spawn_with<'s, 'env, 'scope, P>(
        self,
        scope_ref: &P::ScopeRef<'s, 'env, 'scope>,
    ) -> Self::JoinHandle
    where
        P: ParThreadPool,
        'scope: 's,
        'env: 'scope + 's,
        Self: 'scope + 'env;

    fn join(handle: Self::JoinHandle) -> Self::Output;

    fn run_with_pool<P>(self, pool: &mut P) -> Self::Output
    where
        P: ParThreadPool,
        Self: Sized + 'static,
    {
        let mut handle = None;

        pool.scoped_computation(|scope_ref| {
            handle = Some(self.spawn_with::<P>(&scope_ref));
        });

        Self::join(handle.expect("task queue scope did not produce a join handle"))
    }
}

impl<T: Task + 'static> TaskQueueTrait for TaskSingle<T> {
    type Output = T::Output;
    type JoinHandle = TaskJoin<T::Output>;

    fn run_inline(self) -> Self::Output {
        self.pop().run()
    }

    fn spawn_with<'s, 'env, 'scope, P>(
        self,
        scope_ref: &P::ScopeRef<'s, 'env, 'scope>,
    ) -> Self::JoinHandle
    where
        P: ParThreadPool,
        'scope: 's,
        'env: 'scope + 's,
        Self: 'scope + 'env,
    {
        let join = TaskJoin::new();
        let join_clone = join.clone();
        let task = Arc::new(Mutex::new(Some(self.pop())));

        P::run_in_scope(scope_ref, move || {
            let task = task
                .lock()
                .expect("poisoned task queue lock")
                .take()
                .expect("task queue item executed more than once");
            join_clone.store(task.run());
        });

        join
    }

    fn join(handle: Self::JoinHandle) -> Self::Output {
        handle.join()
    }
}

impl<F, B> TaskQueueTrait for TaskQueue<F, B>
where
    F: Task + 'static,
    B: TaskQueueTrait + StQueue + 'static,
{
    type Output = (F::Output, B::Output);
    type JoinHandle = (TaskJoin<F::Output>, B::JoinHandle);

    fn run_inline(self) -> Self::Output {
        let (front, back) = self.pop();
        (front.run(), back.run_inline())
    }

    fn spawn_with<'s, 'env, 'scope, P>(
        self,
        scope_ref: &P::ScopeRef<'s, 'env, 'scope>,
    ) -> Self::JoinHandle
    where
        P: ParThreadPool,
        'scope: 's,
        'env: 'scope + 's,
        Self: 'scope + 'env,
    {
        let (front, back) = self.pop();

        let front_join = TaskJoin::new();
        let front_join_clone = front_join.clone();
        let front_task = Arc::new(Mutex::new(Some(front)));

        P::run_in_scope(scope_ref, move || {
            let front_task = front_task
                .lock()
                .expect("poisoned task queue lock")
                .take()
                .expect("task queue item executed more than once");
            front_join_clone.store(front_task.run());
        });

        let back_join = back.spawn_with::<P>(scope_ref);

        (front_join, back_join)
    }

    fn join(handle: Self::JoinHandle) -> Self::Output {
        (handle.0.join(), B::join(handle.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pool;
    use std::time::{Duration, Instant};

    struct Add(i32);
    impl Task for Add {
        type Output = i32;
        fn run(self) -> Self::Output {
            self.0 + 1
        }
    }

    #[test]
    fn test_task_single() {
        let queue = TaskSingle::new(Add(41));
        let result = queue.run_inline();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_task_queue_inline() {
        let queue = TaskSingle::new(Add(1)).push_back(Add(2)).push_back(Add(3));

        let result = queue.run_inline();
        assert_eq!(result, (2, (3, 4)));
    }

    #[test]
    fn test_task_queue_parallel() {
        struct SleepAndReturn(u64);

        impl Task for SleepAndReturn {
            type Output = u64;

            fn run(self) -> Self::Output {
                std::thread::sleep(Duration::from_millis(self.0));
                self.0
            }
        }

        let queue = TaskSingle::new(SleepAndReturn(30)).push_back(SleepAndReturn(30));
        let mut pool = Pool::once(2);

        let start = Instant::now();
        let result = queue.run_with_pool(&mut pool);
        let elapsed = start.elapsed();

        assert_eq!(result, (30, 30));
        assert!(elapsed < Duration::from_millis(55));
    }
}
