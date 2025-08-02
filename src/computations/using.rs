pub trait Using {
    type Item: Send;

    fn create(&mut self, thread_idx: usize) -> Self::Item;
}

pub struct UsingClone<T: Clone + Send>(T);

pub fn using_clone<T: Clone + Send>(value: T) -> UsingClone<T> {
    UsingClone(value)
}

impl<T: Clone + Send> Using for UsingClone<T> {
    type Item = T;

    fn create(&mut self, _: usize) -> T {
        self.0.clone()
    }
}

pub struct UsingFun<F, T>
where
    T: Send,
    F: FnMut(usize) -> T,
{
    fun: F,
}

pub fn using_fun_ignoring_thread_idx<T>(
    mut fun: impl FnMut() -> T,
) -> UsingFun<impl FnMut(usize) -> T, T>
where
    T: Send,
{
    let fun = move |_thread_idx: usize| return fun();
    UsingFun { fun }
}

pub fn using_fun_using_thread_idx<T>(
    fun: impl FnMut(usize) -> T,
) -> UsingFun<impl FnMut(usize) -> T, T>
where
    T: Send,
{
    UsingFun { fun }
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
}
