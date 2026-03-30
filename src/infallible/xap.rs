use crate::infallible::size::{Bin, One, Size};

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Size: Size;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    type Map<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    type Inspect<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send;

    type Filter<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;
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

// temporary

pub struct Fake<I, O>(core::marker::PhantomData<(I, O)>);

impl<I, O> Clone for Fake<I, O> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I, O> Copy for Fake<I, O> {}

unsafe impl<I, O> Send for Fake<I, O> {}

impl<I, O> Xap for Fake<I, O> {
    type I = I;

    type O = O;

    type Size = One;

    type Values = core::iter::Empty<Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values {
        Default::default()
    }

    // transformations

    type Map<Q, H>
        = crate::infallible::xap::Fake<Self::I, Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        todo!()
    }

    type Inspect<H>
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = crate::infallible::xap::Fake<Self::I, Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = crate::infallible::xap::Fake<Self::I, <V as IntoIterator>::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        todo!()
    }
}
