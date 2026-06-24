#![allow(missing_docs)]

use std::boxed::Box;
use std::vec::Vec;

/// A dynamic group of tasks that can accept runtime-supplied work.
///
/// This is the escape hatch for cases where the task set is not known at compile time.
/// Tasks are `Fn() + Send + 'static` and stored in a `Vec`, which incurs one heap allocation per task.
///
/// # Note on Design
///
/// The pool's `run_in_scope` API requires `Fn()` (reusable closures) rather than `FnOnce()`.
/// For most dynamic workloads, this is not a limitation since tasks often need to be reusable.
/// For cases requiring full move semantics, use the static `Task` API or structure your code
/// to clone captured values if needed.
///
/// # Example
///
/// ```ignore
/// use orx_parallel::scope::DynamicGroup;
/// use orx_parallel::Pool;
///
/// let mut group = DynamicGroup::new();
/// group.spawn(|| println!("task 1"));
/// group.spawn(|| println!("task 2"));
///
/// let mut pool = Pool::once(8);
/// pool.scoped_computation(|scope| {
///     group.execute(scope);
/// });
/// ```
pub struct DynamicGroup {
    tasks: Vec<Box<dyn Fn() + Send>>,
}

impl DynamicGroup {
    /// Create a new empty dynamic group.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Add a task to the group.
    ///
    /// Note: Tasks must be `Fn() + Send + 'static` due to the pool's execution model.
    pub fn spawn<F: Fn() + Send + 'static>(&mut self, f: F) {
        self.tasks.push(Box::new(f));
    }

    /// Execute all tasks in the group within a pool's scope.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use orx_parallel::{scope::DynamicGroup, Pool};
    ///
    /// let mut group = DynamicGroup::new();
    /// group.spawn(|| println!("hello from task"));
    ///
    /// let mut pool = Pool::once(4);
    /// pool.scoped_computation(|scope| {
    ///     group.execute(scope);
    /// });
    /// ```
    pub fn execute<P: crate::ParThreadPool>(self, scope_ref: &P::ScopeRef<'_, '_, '_>) {
        for task in self.tasks {
            P::run_in_scope(scope_ref, task);
        }
    }

    /// Returns the number of tasks in this group.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns true if the group has no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for DynamicGroup {
    fn default() -> Self {
        Self::new()
    }
}
