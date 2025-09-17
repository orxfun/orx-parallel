/// A type that can [`create`] a value per thread, which will then be send to the thread,
/// and used mutable by the defined computation.
///
/// [`create`]: crate::using::Using::create
pub trait Using {
    /// Item to be used mutably by each threads used in parallel computation.
    type Item: Send + 'static;

    /// Creates an instance of the variable to be used by the `thread_idx`-th thread.
    fn create(&self, thread_idx: usize) -> Self::Item;

    /// Consumes self and creates exactly one instance of the variable.
    fn into_inner(self) -> Self::Item;
}

/// Using variant that creates instances of each thread by cloning an initial value.
pub struct UsingClone<T: Clone + Send + 'static>(T);

impl<T: Clone + Send + 'static> UsingClone<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone + Send + 'static> Using for UsingClone<T> {
    type Item = T;

    fn create(&self, _: usize) -> T {
        self.0.clone()
    }

    fn into_inner(self) -> Self::Item {
        self.0
    }
}

/// Using variant that creates instances of each thread using a closure.
pub struct UsingFun<F, T>
where
    T: Send + 'static,
    F: Fn(usize) -> T,
{
    fun: F,
}

impl<F, T> UsingFun<F, T>
where
    T: Send + 'static,
    F: Fn(usize) -> T,
{
    pub(crate) fn new(fun: F) -> Self {
        Self { fun }
    }
}

impl<F, T> Using for UsingFun<F, T>
where
    T: Send + 'static,
    F: Fn(usize) -> T,
{
    type Item = T;

    fn create(&self, thread_idx: usize) -> Self::Item {
        (self.fun)(thread_idx)
    }

    fn into_inner(self) -> Self::Item {
        (self.fun)(0)
    }
}
