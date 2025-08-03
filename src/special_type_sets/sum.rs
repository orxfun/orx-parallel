use std::ops::Add;

/// Number that can be summed over.
pub trait Sum<Output> {
    /// Zero.
    fn zero() -> Output;

    /// Maps the number to owned value.
    fn map(a: Self) -> Output;

    /// Maps the number to owned value.
    fn u_map<U>(_: &mut U, a: Self) -> Output;

    /// Returns sum of `a` and `b`.
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
    fn u_map<U>(_: &mut U, a: Self) -> X {
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
    fn u_map<U>(_: &mut U, a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn reduce(a: X, b: X) -> X {
        a + b
    }
}
