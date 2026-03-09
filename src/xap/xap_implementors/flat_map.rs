use crate::xap::count::Count;
use crate::xap::fun::filter::{FnFil, Fs};
use crate::xap::fun::filter_map::{FnFil2, FnFilMap};
use crate::xap::fun::flat_map::{FlatMap, FnFlatMap};
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap, Ms};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_implementors::m2::M2;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};

pub struct FlaMap<X: Xap, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FlatMap<I = X::O>> FlaMap<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FlatMap<I = X::O>> Xap for FlaMap<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Count = <X::Count as Count>::ThenMany;

    type Values<'i>
        = <X::Count as Count>::FlatMap<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        <X::Count as Count>::flat_map(self.x.xap(i), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M2<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M2::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = M2<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M2::new(self, FnIns::new(h))
    }

    type Filter<H>
        = FilMap<Self, FnFil2<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        FilMap::new(self, FnFil2::new(h))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, FnFlatMap::new(h))
    }
}

impl<'a, I: 'a + Clone, X: Xap, G: FlatMap<I = X::O>> XapCloned<'a, I> for FlaMap<X, G>
where
    G::O: IntoIterator<Item = &'a I>,
{
    type Cloned = M2<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M2::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy, X: Xap, G: FlatMap<I = X::O>> XapCopied<'a, I> for FlaMap<X, G>
where
    G::O: IntoIterator<Item = &'a I>,
{
    type Copied = M2<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M2::new(self, FnCopied::new())
    }
}
