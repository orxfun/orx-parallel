use crate::Scope;
use core::marker::PhantomData;

pub struct Tasks;

impl Tasks {
    pub fn new() -> TasksEmpty<impl FnOnce() + Send> {
        TasksEmpty::new(|| {})
    }
}

/// A statically typed queue of tasks to be run in parallel on a [`Scope`].
///
/// Since the queue is typed rather than relying on dynamic dispatch, pushed tasks
/// are stored inline: no object safety, boxing or heap allocation is required.
///
/// Tasks are [`push`]ed one by one, none of which start running immediately;
/// they all start in parallel only when [`run_all`] is called.
///
/// [`push`]: Self::push
/// [`run_all`]: Self::run_all
pub trait TaskQueue {
    /// Type of the typed task queue obtained when the new task is pushed.
    type PushBack<T>: TaskQueue
    where
        T: FnOnce() + Send;

    /// Task in the front of the queue.
    type Front: FnOnce() + Send;

    /// Queue obtained when front of the queue is popped.
    type Back: TaskQueue;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send;

    fn run_in_scope<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
    where
        'scope: 's,
        'env: 'scope + 's,
        Self::Front: 'scope + 'env,
        Self::Back: 'scope + 'env;
}

// empty

pub struct TasksEmpty<F>
where
    F: FnOnce() + Send,
{
    p: PhantomData<F>,
}

impl<F> TasksEmpty<F>
where
    F: FnOnce() + Send,
{
    pub fn new(_do_nothing: F) -> Self {
        Self { p: PhantomData }
    }
}

impl<F> TaskQueue for TasksEmpty<F>
where
    F: FnOnce() + Send,
{
    type PushBack<T>
        = TasksSingle<T>
    where
        T: FnOnce() + Send;

    type Front = F;

    type Back = Self;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send,
    {
        TasksSingle::new(task)
    }

    fn run_in_scope<'s, 'env, 'scope>(self, _scope: impl Scope<'s, 'env, 'scope>) {}
}

// single

pub struct TasksSingle<F>
where
    F: FnOnce() + Send,
{
    front: F,
}

impl<F> TasksSingle<F>
where
    F: FnOnce() + Send,
{
    pub fn new(front: F) -> Self {
        Self { front }
    }
}

impl<F> TaskQueue for TasksSingle<F>
where
    F: FnOnce() + Send,
{
    type PushBack<T>
        = TasksMulti<F, TasksSingle<T>>
    where
        T: FnOnce() + Send;

    type Front = F;

    type Back = Self;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send,
    {
        let back = TasksSingle::new(task);
        TasksMulti::new(self.front, back)
    }

    fn run_in_scope<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
    where
        'scope: 's,
        'env: 'scope + 's,
        Self::Front: 'scope + 'env,
    {
        scope.run(self.front);
    }
}

// pair

pub struct TasksMulti<F, B>
where
    F: FnOnce() + Send,
    B: TaskQueue,
{
    front: F,
    back: B,
}

impl<F, B> TasksMulti<F, B>
where
    F: FnOnce() + Send,
    B: TaskQueue,
{
    pub fn new(front: F, back: B) -> Self {
        Self { front, back }
    }
}

impl<F, B> TaskQueue for TasksMulti<F, B>
where
    F: FnOnce() + Send,
    B: TaskQueue,
{
    type PushBack<T>
        = TasksMulti<F, B::PushBack<T>>
    where
        T: FnOnce() + Send;

    type Front = F;

    type Back = B;

    fn push<T>(self, task: T) -> Self::PushBack<T>
    where
        T: FnOnce() + Send,
    {
        let back = self.back.push(task);
        TasksMulti::new(self.front, back)
    }

    fn run_in_scope<'s, 'env, 'scope>(self, scope: impl Scope<'s, 'env, 'scope>)
    where
        'scope: 's,
        'env: 'scope + 's,
        Self::Front: 'scope + 'env,
        Self::Back: 'scope + 'env,
    {
        let Self { front, back } = self;
        scope.run(front);
        back.run_in_scope(scope);
    }
}
