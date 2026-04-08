use crate::infallible::fun::Map;
use crate::infallible::sizes::SizeInf;
use crate::sizes::{Bin, One, Size};

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Size: SizeInf;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    fn map<Q, H>(self, h: H) -> MapOf<Self, Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        <Self::Size as SizeInf>::map(self, h)
    }

    fn inspect<H>(self, h: H) -> InsOf<Self, H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        <Self::Size as SizeInf>::inspect(self, h)
    }

    fn filter<H>(self, h: H) -> FilOf<Self, H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        <Self::Size as SizeInf>::filter(self, h)
    }

    fn filter_map<Q, H>(self, h: H) -> FilMapOf<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        <Self::Size as SizeInf>::filter_map(self, h)
    }

    fn flat_map<V, H>(self, h: H) -> FlatMapOf<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        <Self::Size as SizeInf>::flat_map(self, h)
    }

    // transformations - helper

    fn mapped<M>(self, m: M) -> MappedOf<Self, M>
    where
        M: Map<I = Self::O>,
    {
        <Self::Size as SizeInf>::mapped(self, m)
    }
}

// one

pub trait XapOne: Xap<Size = One> {
    #[inline(always)]
    fn one_value(&self, i: Self::I) -> Self::O {
        // SAFETY: by definition the result has exactly one element
        unsafe { self.xap(i).into_iter().next().unwrap_unchecked() }
    }
}

impl<X: Xap<Size = One>> XapOne for X {}

// bin

pub trait XapBin: Xap<Size = Bin> {
    #[inline(always)]
    fn bin_value(&self, i: Self::I) -> Option<Self::O> {
        // SAFETY: by definition the result has exactly zero or one element
        self.xap(i).into_iter().next()
    }
}

impl<X: Xap<Size = Bin>> XapBin for X {}

// helper types

pub type MapOf<X, Q, H> = <<X as Xap>::Size as SizeInf>::Map<X, Q, H>;

pub type InsOf<X, H> = <<X as Xap>::Size as SizeInf>::Inspect<X, H>;

pub type FilOf<X, H> = <<X as Xap>::Size as SizeInf>::Filter<X, H>;

pub type FilMapOf<X, Q, H> = <<X as Xap>::Size as SizeInf>::FilterMap<X, Q, H>;

pub type FlatMapOf<X, V, H> = <<X as Xap>::Size as SizeInf>::FlatMap<X, V, H>;

pub type MappedOf<X, M> = <<X as Xap>::Size as SizeInf>::Mapped<X, M>;
