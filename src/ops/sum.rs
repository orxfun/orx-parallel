use core::ops::Add;

/// Number that can be summed over.
pub trait Sum<Output> {
    /// Zero.
    fn zero() -> Output;

    /// Maps the number to owned value.
    fn owned(a: Self) -> Output;

    /// Maps the number to owned value.
    fn u_owned<U>(_: &mut U, a: Self) -> Output;

    /// Returns sum of `a` and `b`.
    fn add(a: Output, b: Output) -> Output;

    /// Returns sum of `a` and `b`.
    fn u_add<U>(_: &mut U, a: Output, b: Output) -> Output;
}

impl<X> Sum<X> for X
where
    X: Default + Add<X, Output = X>,
{
    fn zero() -> X {
        X::default()
    }

    #[inline(always)]
    fn owned(a: Self) -> X {
        a
    }

    #[inline(always)]
    fn u_owned<U>(_: &mut U, a: Self) -> X {
        a
    }

    #[inline(always)]
    fn add(a: X, b: X) -> X {
        a + b
    }

    #[inline(always)]
    fn u_add<U>(_: &mut U, a: X, b: X) -> X {
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
    fn owned(a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn u_owned<U>(_: &mut U, a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn add(a: X, b: X) -> X {
        a + b
    }

    #[inline(always)]
    fn u_add<U>(_: &mut U, a: X, b: X) -> X {
        a + b
    }
}

impl<'a, X> Sum<X> for &'a mut X
where
    X: Default + Add<X, Output = X> + Copy,
    &'a X: Add<&'a X, Output = X>,
{
    fn zero() -> X {
        X::default()
    }

    #[inline(always)]
    fn owned(a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn u_owned<U>(_: &mut U, a: Self) -> X {
        *a
    }

    #[inline(always)]
    fn add(a: X, b: X) -> X {
        a + b
    }

    #[inline(always)]
    fn u_add<U>(_: &mut U, a: X, b: X) -> X {
        a + b
    }
}
