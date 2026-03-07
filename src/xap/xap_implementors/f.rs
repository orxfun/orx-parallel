use crate::xap::count::One;
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_implementors::xap_iters::IterF;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct F<X: Xap, G: Fn(&X::O) -> bool> {
    x: X,
    g: G,
}

impl<X: Xap, G: Fn(&X::O) -> bool> F<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: Fn(&X::O) -> bool> Xap for F<X, G> {
    type I = X::I;

    type O = X::O;

    type Count = One;

    type Values<'i>
        = IterF<IterOf<'i, X>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        IterF::new(self.x.xap(i).into_iter(), &self.g)
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
