use std::ops::Add;

pub trait Sum<Output> {
    fn zero() -> Output;

    fn map(a: Self) -> Output;

    fn reduce(a: Output, b: Output) -> Output;
}

impl<X> Sum<X> for X
where
    X: Default + Add<X, Output = X>,
{
    fn zero() -> X {
        X::default()
    }

    #[inline(always)]
    fn map(a: Self) -> X {
        a
    }

    #[inline(always)]
    fn reduce(a: X, b: X) -> X {
        a + b
    }
}

impl<'a, X> Sum<X> for &'a X
where
    X: Default + Add<X, Output = X> + Copy,
    &'a X: Add<&'a X, Output = X>,
{
    fn zero() -> X {
        X::default()
    }

    #[inline(always)]
    fn map(a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn reduce(a: X, b: X) -> X {
        a + b
    }
}
