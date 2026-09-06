use crate::Scope;
use orx_meta::queue;

/// Entry point for building a statically typed [`TaskQueue`] to run in parallel
/// via [`ThreadPool::run_all`].
///
/// Since the queue is typed rather than relying on dynamic dispatch, pushed tasks
/// are stored inline: no object safety, boxing or heap allocation is required.
///
/// [`ThreadPool::run_all`]: crate::ThreadPool::run_all
///
/// # Example
///
/// ```rust
/// use orx_parallel::*;
///
/// let work_for = |n| std::thread::sleep(std::time::Duration::from_millis(n));
///
/// let tasks = Tasks::new()
///     .push(|| {
///         work_for(90);
///         println!("t1 completes 4th");
///     })
///     .push(|| println!("t2 completes 1st"))
///     .push(|| {
///         work_for(10);
///         println!("t3 completes 2nd");
///     })
///     .push(|| {
///         work_for(50);
///         println!("t4 completes 3rd");
///     });
///
/// Pool::global().run_all(tasks);
///
/// // prints:
/// // t2 completes 1st
/// // t3 completes 2nd
/// // t4 completes 3rd
/// // t1 completes 4th
/// ```
///
/// Below is a more practical example: computing independent statistics over the same
/// input concurrently and collecting the results:
///
/// ```rust
/// use orx_parallel::*;
/// use std::sync::Mutex;
///
/// let numbers = [4, 8, 15, 16, 23, 42];
///
/// let sum = Mutex::new(0);
/// let max = Mutex::new(i32::MIN);
/// let all_positive = Mutex::new(false);
///
/// let tasks = tasks![
///     || *sum.lock().unwrap() = numbers.iter().sum(),
///     || *max.lock().unwrap() = numbers.iter().copied().max().unwrap(),
///     || *all_positive.lock().unwrap() = numbers.iter().all(|&x| x > 0),
/// ];
///
/// Pool::global().run_all(tasks);
///
/// println!(
///     "sum={}, max={}, all_positive={}",
///     sum.into_inner().unwrap(),
///     max.into_inner().unwrap(),
///     all_positive.into_inner().unwrap(),
/// );
/// ```
///
/// Tasks can also be built fluently via [`Tasks::new`] and [`TaskQueue::push`].
pub struct Tasks;

impl Tasks {
    /// Creates a new, empty task queue to [`push`] tasks onto.
    ///
    /// [`push`]: TaskQueue::push
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TasksEmpty {
        TasksEmpty::new()
    }
}

/// Macro helper to build a statically typed [`TaskQueue`] with the given tasks.
///
/// Returns a task queue (equivalent to chaining [`Tasks::new().push(...)`](Tasks::new)).
///
/// # Example
///
/// ```rust
/// use orx_parallel::*;
/// use std::sync::Mutex;
///
/// let numbers = [4, 8, 15, 16, 23, 42];
///
/// let sum = Mutex::new(0);
/// let max = Mutex::new(i32::MIN);
/// let all_positive = Mutex::new(false);
///
/// let tasks = tasks![
///     || *sum.lock().unwrap() = numbers.iter().sum(),
///     || *max.lock().unwrap() = numbers.iter().copied().max().unwrap(),
///     || *all_positive.lock().unwrap() = numbers.iter().all(|&x| x > 0),
/// ];
///
/// Pool::global().run_all(tasks);
///
/// assert_eq!(*sum.lock().unwrap(), 108);
/// assert_eq!(*max.lock().unwrap(), 42);
/// assert!(*all_positive.lock().unwrap());
/// ```
#[macro_export]
macro_rules! tasks {
    () => {
        $crate::Tasks::new()
    };
    ( $( $task:expr ),* $(,)? ) => {
        $crate::Tasks::new()
            $( .push($task) )*
    };
}

#[queue(TaskQueue; TasksEmpty, TasksSingle, TasksMulti)]
pub trait ParFun {
    fn run<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
    where
        'scope: 's,
        'env: 'scope + 's,
        Self: 'scope + 'env;
}

impl<F: FnOnce() + Send> ParFun for F {
    #[inline]
    fn run<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
    where
        'scope: 's,
        'env: 'scope + 's,
        Self: 'scope + 'env,
    {
        scope.run(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_tasks_macro() {
        let counter = AtomicUsize::new(0);

        let t0 = tasks![];
        Pool::global().run_all(t0);
        assert_eq!(counter.load(Ordering::Relaxed), 0);

        let t1 = tasks![|| {
            counter.fetch_add(1, Ordering::Relaxed);
        }];
        Pool::global().run_all(t1);
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        let t3 = tasks![
            || {
                counter.fetch_add(10, Ordering::Relaxed);
            },
            || {
                counter.fetch_add(100, Ordering::Relaxed);
            },
            || {
                counter.fetch_add(1000, Ordering::Relaxed);
            },
        ];
        Pool::global().run_all(t3);
        assert_eq!(counter.load(Ordering::Relaxed), 1111);
    }
}
