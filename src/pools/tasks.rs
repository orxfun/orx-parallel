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
/// let tasks = Tasks::new()
///     .push(|| *sum.lock().unwrap() = numbers.iter().sum())
///     .push(|| *max.lock().unwrap() = numbers.iter().copied().max().unwrap())
///     .push(|| *all_positive.lock().unwrap() = numbers.iter().all(|&x| x > 0));
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

// impl<F: ParFun> ParFun for TasksSingle<F> {
//     fn run<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
//     where
//         'scope: 's,
//         'env: 'scope + 's,
//         Self: 'scope + 'env,
//     {
//         self.f.run(scope);
//     }
// }

// impl<F: ParFun, B: TaskQueue> ParFun for TasksMulti<F, B> {
//     fn run<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
//     where
//         'scope: 's,
//         'env: 'scope + 's,
//         Self: 'scope + 'env,
//     {
//         self.f.run(scope);
//         self.b.run(scope);
//     }
// }
