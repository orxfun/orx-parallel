use crate::xap::count::One;
use crate::xap::fun::filter::{FWr, Fs};
use crate::xap::fun::map::{MWr, Ms};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct FlaMap<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> {
    x: X,
    g: G,
}

impl<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> FlaMap<X, O, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> Xap for FlaMap<X, O, G> {
    type I = X::I;

    type O = O::Item;

    type Count = One;

    type Values<'i>
        = core::iter::FlatMap<IterOf<'i, X>, O, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        self.x.xap(i).into_iter().flat_map(&self.g)
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
        let h = MWr::new(h);
        M::new(self, Ms::new(h))
    }

    type Inspect<H>
        = Ins<Self, H>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        Ins::new(self, h)
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
        = FilMap<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, h)
    }

    type FlatMap<V, H>
        = FlaMap<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, h)
    }
}
