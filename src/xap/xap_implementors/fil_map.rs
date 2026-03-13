use crate::xap::count::{Count, One};
use crate::xap::fun::filter::{FnFil, Fs};
use crate::xap::fun::filter_map::{FilterMap, FnFil2, FnFilMap};
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap, Ms};
use crate::xap::xap_implementors::flat_map::FlaMap;
use crate::xap::xap_implementors::m2::M2;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};

pub struct FilMap<X: Xap, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterMap<I = X::O>> FilMap<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterMap<I = X::O>> Xap for FilMap<X, G> {
    type I = X::I;

    type O = G::O;

    type Count = <X::Count as Count>::ThenZeroOne;

    type Values = <X::Count as Count>::FilterMap<X::Values, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        <X::Count as Count>::filter_map(self.x.xap(i), self.g)
    }

    // transformations

    type Map<Q, H>
        = M2<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy,
    {
        M2::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = M2<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy,
    {
        M2::new(self, FnIns::new(h))
    }

    type Filter<H>
        = FilMap<Self, FnFil2<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy,
    {
        FilMap::new(self, FnFil2::new(h))
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

impl<'a, I: 'a + Clone, X: Xap, G: FilterMap<I = X::O, O = &'a I>> XapCloned<'a, I>
    for FilMap<X, G>
{
    type Cloned = M2<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M2::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy, X: Xap, G: FilterMap<I = X::O, O = &'a I>> XapCopied<'a, I>
    for FilMap<X, G>
{
    type Copied = M2<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M2::new(self, FnCopied::new())
    }
}
