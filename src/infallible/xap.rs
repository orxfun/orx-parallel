use crate::infallible::fun::Map;
use crate::sizes::Size;
use crate::sizes::{Bin, One};

/// Transformation from one input to zero, one, or many output values.
pub trait Xap: Copy + Send {
    /// Input item type.
    type I;

    /// Output item type.
    type O;

    /// Output cardinality marker.
    type Size: Size;

    /// Iterator-like container of output values.
    type Values: IntoIterator<Item = Self::O>;

    /// Applies the transformation to one input value.
    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    /// Maps each output value.
    fn map<Q, H>(self, h: H) -> MapOf<Self, Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        <Self::Size as Size>::map(self, h)
    }

    /// Inspects each output value.
    fn inspect<H>(self, h: H) -> InsOf<Self, H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        <Self::Size as Size>::inspect(self, h)
    }

    /// Filters output values.
    fn filter<H>(self, h: H) -> FilOf<Self, H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        <Self::Size as Size>::filter(self, h)
    }

    /// Maps and optionally filters output values.
    fn filter_map<Q, H>(self, h: H) -> FilMapOf<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        <Self::Size as Size>::filter_map(self, h)
    }

    /// Expands each output value into more values.
    fn flat_map<V, H>(self, h: H) -> FlatMapOf<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        <Self::Size as Size>::flat_map(self, h)
    }

    /// Flattens nested output values.
    fn flatten(self) -> FlattenOf<Self>
    where
        Self::O: IntoIterator,
    {
        <Self::Size as Size>::flatten(self)
    }

    // transformations - helper

    /// Applies a map implementation object to each output value.
    fn mapped<M>(self, m: M) -> MappedOf<Self, M>
    where
        M: Map<I = Self::O>,
    {
        <Self::Size as Size>::mapped(self, m)
    }
}

// one

/// Convenience methods for xaps that always yield exactly one value.
pub trait XapOne: Xap<Size = One> {
    #[inline(always)]
    /// Returns the single output value.
    fn one_value(&self, i: Self::I) -> Self::O {
        // SAFETY: by definition the result has exactly one element
        unsafe { self.xap(i).into_iter().next().unwrap_unchecked() }
    }
}

impl<X: Xap<Size = One>> XapOne for X {}

// bin

/// Convenience methods for xaps that yield zero or one value.
pub trait XapBin: Xap<Size = Bin> {
    #[inline(always)]
    /// Returns the optional output value.
    fn bin_value(&self, i: Self::I) -> Option<Self::O> {
        // SAFETY: by definition the result has exactly zero or one element
        self.xap(i).into_iter().next()
    }
}

impl<X: Xap<Size = Bin>> XapBin for X {}

// helper types

/// Resulting xap type after `map`.
pub type MapOf<X, Q, H> = <<X as Xap>::Size as Size>::Map<X, Q, H>;

/// Resulting xap type after `inspect`.
pub type InsOf<X, H> = <<X as Xap>::Size as Size>::Inspect<X, H>;

/// Resulting xap type after `filter`.
pub type FilOf<X, H> = <<X as Xap>::Size as Size>::Filter<X, H>;

/// Resulting xap type after `filter_map`.
pub type FilMapOf<X, Q, H> = <<X as Xap>::Size as Size>::FilterMap<X, Q, H>;

/// Resulting xap type after `flat_map`.
pub type FlatMapOf<X, V, H> = <<X as Xap>::Size as Size>::FlatMap<X, V, H>;

/// Resulting xap type after `flatten`.
pub type FlattenOf<X> = <<X as Xap>::Size as Size>::Flatten<X>;

/// Resulting xap type after `mapped`.
pub type MappedOf<X, M> = <<X as Xap>::Size as Size>::Mapped<X, M>;
