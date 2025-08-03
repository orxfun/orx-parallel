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

// TODO: new using transformations
#[allow(dead_code)]
pub fn using_fun_ignoring_thread_idx<T>(
    mut fun: impl FnMut() -> T,
) -> UsingFun<impl FnMut(usize) -> T, T>
where
    T: Send,
{
    let fun = move |_thread_idx: usize| fun();
    UsingFun { fun }
}

#[allow(dead_code)]
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

    fn into_inner(mut self) -> Self::Item {
        (self.fun)(0)
    }
}
