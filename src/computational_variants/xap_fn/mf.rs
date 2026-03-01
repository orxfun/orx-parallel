use core::marker::PhantomData;

pub struct MF<I, O, X, Y>
where
    X: Fn(I) -> O,
    Y: Fn(&O) -> bool,
{
    m: X,
    f: Y,
    p: PhantomData<I>,
}

impl<I, O, X, Y> MF<I, O, X, Y>
where
    X: Fn(I) -> O,
    Y: Fn(&O) -> bool,
{
    pub fn new(m: X, f: Y) -> Self {
        let p = PhantomData;
        Self { m, f, p }
    }
}
