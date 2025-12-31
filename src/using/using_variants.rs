/// A type that can [`create`] a value per thread, which will then be send to the thread,
/// and used mutable by the defined computation.
///
/// [`create`]: crate::using::Using::create
pub trait Using<'using>: Sync {
    /// Item to be used mutably by each threads used in parallel computation.
    type Item: 'using;

    /// Creates an instance of the variable to be used by the `thread_idx`-th thread.
    fn create(&self, thread_idx: usize) -> Self::Item;

    /// Consumes self and creates exactly one instance of the variable.
    fn into_inner(self) -> Self::Item;
}

/// Using variant that creates instances of each thread by cloning an initial value.
pub struct UsingClone<T: Clone + 'static>(T);

impl<T: Clone + 'static> UsingClone<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone + 'static> Using<'static> for UsingClone<T> {
    type Item = T;

    fn create(&self, _: usize) -> T {
        self.0.clone()
    }

    fn into_inner(self) -> Self::Item {
        self.0
    }
}

unsafe impl<T: Clone + 'static> Sync for UsingClone<T> {}

/// Using variant that creates instances of each thread using a closure.
pub struct UsingFun<F, T>
where
    F: Fn(usize) -> T + Sync,
{
    fun: F,
}

impl<F, T> UsingFun<F, T>
where
    F: Fn(usize) -> T + Sync,
{
    pub(crate) fn new(fun: F) -> Self {
        Self { fun }
    }
}

impl<'using, F, T> Using<'using> for UsingFun<F, T>
where
    T: 'using,
    F: Fn(usize) -> T + Sync,
{
    type Item = T;

    fn create(&self, thread_idx: usize) -> Self::Item {
        (self.fun)(thread_idx)
    }

    fn into_inner(self) -> Self::Item {
        (self.fun)(0)
    }
}
