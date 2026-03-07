use crate::xap::F;
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::Xap;
use core::marker::PhantomData;

pub struct F0<I, G: Fn(&I) -> bool> {
    g: G,
    p: PhantomData<I>,
}

impl<I, G: Fn(&I) -> bool> F0<I, G> {
    pub fn new(g: G) -> Self {
        let p = PhantomData;
        Self { g, p }
    }
}

impl<I, G: Fn(&I) -> bool> Xap for F0<I, G> {
    type I = I;

    type O = I;

    type Values<'i>
        = Option<I>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        match (self.g)(&i) {
            true => Some(i),
            false => None,
        }
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
