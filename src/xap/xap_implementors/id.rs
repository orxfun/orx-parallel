use crate::xap::count::One;
use crate::xap::fun::filter::{FWr, Fs};
use crate::xap::fun::filter_map::FilMWr;
use crate::xap::fun::flat_map::FlaMWr;
use crate::xap::fun::map::{InsWr, MWr, Ms};
use crate::xap::xap_implementors::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::Xap;
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Count = One;

    type Values<'i>
        = [I; 1]
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        [i]
    }

    // transformations

    type Map<Q, H>
        = M<Self, Ms<MWr<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, Ms::new(MWr::new(h)))
    }

    type Inspect<H>
        = M<Self, Ms<InsWr<Self::O, H>>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M::new(self, Ms::new(InsWr::new(h)))
    }

    type Filter<H>
        = F<Self, Fs<FWr<Self::O, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, Fs::new(FWr::new(h)))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FilMWr<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, FilMWr::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FlaMWr<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, FlaMWr::new(h))
    }
}
