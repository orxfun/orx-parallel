use crate::infallible_use::fun::Map;
use crate::infallible_use::sizes::SizeInfUse;
use crate::sizes::{Bin, One};

pub trait XapUse: Copy + Send {
    type U;

    type I;

    type O;

    type Size: SizeInfUse;

    type Values: IntoIterator<Item = Self::O>;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values;

    // transformations

    fn map<Q, H>(self, h: H) -> MapOf<Self, Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        <Self::Size as SizeInfUse>::map(self, h)
    }

    fn inspect<H>(self, h: H) -> InsOf<Self, H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        <Self::Size as SizeInfUse>::inspect(self, h)
    }

    fn filter<H>(self, h: H) -> FilOf<Self, H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        <Self::Size as SizeInfUse>::filter(self, h)
    }

    fn filter_map<Q, H>(self, h: H) -> FilMapOf<Self, Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        <Self::Size as SizeInfUse>::filter_map(self, h)
    }

    fn flat_map<V, H>(self, h: H) -> FlatMapOf<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        <Self::Size as SizeInfUse>::flat_map(self, h)
    }

    // transformations - helper

    fn mapped<M>(self, m: M) -> MappedOf<Self, M>
    where
        M: Map<U = Self::U, I = Self::O>,
    {
        <Self::Size as SizeInfUse>::mapped(self, m)
    }
}

// one

pub trait XapUseOne: XapUse<Size = One> {
    #[inline(always)]
    fn one_value(&self, u: *mut Self::U, i: Self::I) -> Self::O {
        // SAFETY: by definition the result has exactly one element
        unsafe { self.xap_use(u, i).into_iter().next().unwrap_unchecked() }
    }
}

impl<X: XapUse<Size = One>> XapUseOne for X {}

// bin

pub trait XapUseBin: XapUse<Size = Bin> {
    #[inline(always)]
    fn bin_value(&self, u: *mut Self::U, i: Self::I) -> Option<Self::O> {
        // SAFETY: by definition the result has exactly zero or one element
        self.xap_use(u, i).into_iter().next()
    }
}

impl<X: XapUse<Size = Bin>> XapUseBin for X {}

// // helper types

pub type MapOf<X, Q, H> = <<X as XapUse>::Size as SizeInfUse>::Map<X, Q, H>;

pub type InsOf<X, H> = <<X as XapUse>::Size as SizeInfUse>::Inspect<X, H>;

pub type FilOf<X, H> = <<X as XapUse>::Size as SizeInfUse>::Filter<X, H>;

pub type FilMapOf<X, Q, H> = <<X as XapUse>::Size as SizeInfUse>::FilterMap<X, Q, H>;

pub type FlatMapOf<X, V, H> = <<X as XapUse>::Size as SizeInfUse>::FlatMap<X, V, H>;

pub type MappedOf<X, M> = <<X as XapUse>::Size as SizeInfUse>::Mapped<X, M>;
