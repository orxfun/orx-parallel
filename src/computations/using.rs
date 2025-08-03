pub trait Using {
    type Item: Send;

    fn create(&mut self, thread_idx: usize) -> Self::Item;

    fn into_inner(self) -> Self::Item;
}

pub struct UsingClone<T: Clone + Send>(T);

impl<T: Clone + Send> UsingClone<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone + Send> Using for UsingClone<T> {
    type Item = T;

    fn create(&mut self, _: usize) -> T {
        self.0.clone()
    }

    fn into_inner(self) -> Self::Item {
        self.0
    }
}

pub struct UsingFun<F, T>
where
    T: Send,
    F: FnMut(usize) -> T,
{
    fun: F,
}

impl<F, T> UsingFun<F, T>
where
    T: Send,
    F: FnMut(usize) -> T,
{
    pub fn new(fun: F) -> Self {
        Self { fun }
    }
}

impl<F, T> Using for UsingFun<F, T>
where
    T: Send,
    F: FnMut(usize) -> T,
{
    type Item = T;

    fn create(&mut self, thread_idx: usize) -> Self::Item {
        (self.fun)(thread_idx)
    }

    fn into_inner(mut self) -> Self::Item {
        (self.fun)(0)
    }
}
