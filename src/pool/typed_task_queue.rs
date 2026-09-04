use crate::Scope;
use core::marker::PhantomData;

pub trait TaskQueue<'s, 'env, 'scope>
where
    'scope: 's,
    'env: 'scope + 's,
{
    /// Type of the typed task queue obtained when the new task is pushed.
    type PushBack<T>: TaskQueue<'s, 'env, 'scope>
    where
        T: FnOnce() + Send + 'scope + 'env;

    /// Task in the front of the queue.
    type Front: FnOnce() + Send + 'scope + 'env;

    /// Queue obtained when front of the queue is popped.
    type Back: TaskQueue<'s, 'env, 'scope>;

    /// Pushes the `task` and returns the new typed task queue.
    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send + 'scope + 'env;

    /// Runs all tasks in the queue in parallel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let work_for = |n| std::thread::sleep(std::time::Duration::from_millis(n));
    ///
    /// global_pool().scope(|s| {
    ///     s.tasks()
    ///         .push(|| {
    ///             work_for(90);
    ///             println!("t1 completes 4th");
    ///         })
    ///         .push(|| println!("t2 completes 1st"))
    ///         .push(|| {
    ///             work_for(10);
    ///             println!("t3 completes 2nd");
    ///         })
    ///         .push(|| {
    ///             work_for(50);
    ///             println!("t4 completes 3rd");
    ///         })
    ///         .run_all();
    /// });
    /// ```
    fn run_all(self);
}

// empty

pub struct TasksEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
{
    scope: S,
    p: PhantomData<&'s &'env &'scope F>,
}

impl<'s, 'env, 'scope, S, F> TasksEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    pub fn new(scope: S, _do_nothing: F) -> Self {
        Self {
            scope,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F> TaskQueue<'s, 'env, 'scope> for TasksEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    type PushBack<T>
        = TasksSingle<'s, 'env, 'scope, S, T>
    where
        T: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = Self;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send + 'scope + 'env,
    {
        TasksSingle::new(self.scope, task)
    }

    fn run_all(self) {}
}

// single

pub struct TasksSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    scope: S,
    front: F,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F> TasksSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    pub fn new(scope: S, front: F) -> Self {
        Self {
            scope,
            front,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F> TaskQueue<'s, 'env, 'scope>
    for TasksSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    type PushBack<T>
        = TasksMulti<'s, 'env, 'scope, S, F, TasksSingle<'s, 'env, 'scope, S, T>>
    where
        T: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = Self;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send + 'scope + 'env,
    {
        let back = TasksSingle::new(self.scope, task);
        TasksMulti::new(self.scope, self.front, back)
    }

    fn run_all(self) {
        let (scope, work) = (self.scope, self.front);
        scope.run(work);
    }
}

// pair

pub struct TasksMulti<'s, 'env, 'scope, S, F, B>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: TaskQueue<'s, 'env, 'scope>,
{
    scope: S,
    front: F,
    back: B,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F, B> TasksMulti<'s, 'env, 'scope, S, F, B>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: TaskQueue<'s, 'env, 'scope>,
{
    pub fn new(scope: S, front: F, back: B) -> Self {
        Self {
            scope,
            front,
            back,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F, B> TaskQueue<'s, 'env, 'scope>
    for TasksMulti<'s, 'env, 'scope, S, F, B>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: TaskQueue<'s, 'env, 'scope>,
{
    type PushBack<T>
        = TasksMulti<'s, 'env, 'scope, S, F, B::PushBack<T>>
    where
        T: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = B;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send + 'scope + 'env,
    {
        let back = self.back.push(task);
        TasksMulti::new(self.scope, self.front, back)
    }

    fn run_all(self) {
        let (scope, work, remaining) = (self.scope, self.front, self.back);
        scope.run(work);
        remaining.run_all();
    }
}

#[cfg(test)]
#[test]
fn abc() {
    use crate::{ThreadPool, global_pool};
    use core::num::NonZeroUsize;
    use core::time::Duration;
    use std::*;

    let work_for = |n| std::thread::sleep(std::time::Duration::from_millis(n));

    global_pool().scope(|s| {
        s.tasks()
            .push(|| {
                work_for(90);
                println!("t1 completes 4th");
            })
            .push(|| println!("t2 completes 1st"))
            .push(|| {
                work_for(10);
                println!("t3 completes 2nd");
            })
            .push(|| {
                work_for(50);
                println!("t4 completes 3rd");
            })
            .run_all();
    });

    assert_eq!(global_pool().max_num_threads(), NonZeroUsize::MAX);
}
