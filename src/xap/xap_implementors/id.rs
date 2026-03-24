use crate::xap::FlaMap;
use crate::xap::count::One;
use crate::xap::fun::filter_map::{FnFil, FnFilMap};
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap};
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Clone for Id<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Id<I> {}

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Count = One;

    type Values = [I; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [i]
    }

    #[inline(always)]
    fn into_iter_over(
        self,
        inputs: impl IntoIterator<Item = Self::I>,
    ) -> impl Iterator<Item = Self::O> {
        inputs.into_iter()
    }

    // transformations

    type Map<Q, H>
        = M<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy,
    {
        M::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = M<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy,
    {
        M::new(self, FnIns::new(h))
    }

    type Filter<H>
        = FilMap<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy,
    {
        FilMap::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy,
    {
        FilMap::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy,
    {
        FlaMap::new(self, FnFlatMap::new(h))
    }
}

impl<'a, I: 'a + Clone> XapCloned<'a, I> for Id<&'a I> {
    type Cloned = M<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy> XapCopied<'a, I> for Id<&'a I> {
    type Copied = M<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M::new(self, FnCopied::new())
    }
}
