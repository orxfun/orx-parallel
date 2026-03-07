use crate::xap::count::One;
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct FlaM<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> {
    x: X,
    g: G,
}

impl<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> FlaM<X, O, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, O: IntoIterator, G: Fn(X::O) -> O> Xap for FlaM<X, O, G> {
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
        = M<Self, Q, H>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, h)
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
        = F<Self, H>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, h)
    }

    type FilterMap<Q, H>
        = FilM<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilM::new(self, h)
    }

    type FlatMap<V, H>
        = FlaM<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaM::new(self, h)
    }
}
