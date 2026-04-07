use crate::infallible::fun::Map;
use crate::infallible::sizes::{Bin, Many, One, Size};

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Size: Size;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    fn map<Q, H>(self, h: H) -> <Self::Size as Size>::Map<Self, Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        <Self::Size as Size>::map(self, h)
    }

    type Inspect<H>: Xap<I = Self::I, O = Self::O, Size = Self::Size>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send;

    type Filter<H>: Xap<I = Self::I, O = Self::O, Size = <Self::Size as Size>::ThenBin>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: Xap<I = Self::I, O = Q, Size = <Self::Size as Size>::ThenBin>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: Xap<I = Self::I, O = V::Item, Size = Many>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<M>: Xap<I = Self::I, O = M::O, Size = Self::Size>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>;
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

pub type MapOf<X, Q, H> = <<X as Xap>::Size as Size>::Map<X, Q, H>;

pub type InsOf<X, H> = <<X as Xap>::Size as Size>::Inspect<X, H>;

pub type FilOf<X, H> = <<X as Xap>::Size as Size>::Filter<X, H>;

pub type FilMapOf<X, Q, H> = <<X as Xap>::Size as Size>::FilterMap<X, Q, H>;

pub type FlatMapOf<X, V, H> = <<X as Xap>::Size as Size>::FlatMap<X, V, H>;

pub type MappedOf<X, M> = <<X as Xap>::Size as Size>::Mapped<X, M>;
