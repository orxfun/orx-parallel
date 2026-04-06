use crate::infallible::size::{Bin, Many, One, Size};
use crate::infallible_using::fun::Map;

pub trait Xap: Copy + Send {
    type I;

    type O;

    type U;

    type Size: Size;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values;

    // transformations

    type Map<Q, H>: Xap<U = Self::U, I = Self::I, O = Q, Size = Self::Size>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    type Inspect<H>: Xap<U = Self::U, I = Self::I, O = Self::O, Size = Self::Size>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    type Filter<H>: Xap<U = Self::U, I = Self::I, O = Self::O, Size = <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: Xap<U = Self::U, I = Self::I, O = Q, Size = <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: Xap<U = Self::U, I = Self::I, O = V::Item, Size = Many>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<M>: Xap<U = Self::U, I = Self::I, O = M::O, Size = Self::Size>
    where
        M: Map<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<U = Self::U, I = Self::O>;
}

// one

pub trait XapOne: Xap<Size = One> {
    #[inline(always)]
    fn one_value(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        // SAFETY: by definition the result has exactly one element
        unsafe { self.xap(u, i).into_iter().next().unwrap_unchecked() }
    }
}

impl<X: Xap<Size = One>> XapOne for X {}

// bin

pub trait XapBin: Xap<Size = Bin> {
    #[inline(always)]
    fn bin_value(&self, u: &mut Self::U, i: Self::I) -> Option<Self::O> {
        // SAFETY: by definition the result has exactly zero or one element
        self.xap(u, i).into_iter().next()
    }
}

impl<X: Xap<Size = Bin>> XapBin for X {}
