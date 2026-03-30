use crate::infallible_arch::count::Count;
use crate::infallible_arch::fun::filter_map::{FnFil, FnFilMap};
use crate::infallible_arch::fun::flat_map::FnFlatMap;
use crate::infallible_arch::fun::map::{FnCloned, FnCopied, FnIns, FnMap, Map};
use crate::infallible_arch::xap::{Xap, XapCloned, XapCopied};
use crate::infallible_arch::xap_variants::fil_map::FilMap;
use crate::infallible_arch::xap_variants::flat_map::FlaMap;

pub struct M<X: Xap, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: Map<I = X::O>> Clone for M<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap, G: Map<I = X::O>> Copy for M<X, G> {}

impl<X: Xap, G: Map<I = X::O>> M<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: Map<I = X::O>> Xap for M<X, G> {
    type I = X::I;

    type O = G::O;

    type Count = <X::Count as Count>::ThenOne;

    type Values = <X::Count as Count>::Map<X::Values, G>;

    fn xap(&self, i: Self::I) -> Self::Values {
        <X::Count as Count>::map(self.x.xap(i), self.g)
    }

    #[inline(always)]
    fn into_iter_over(
        self,
        inputs: impl IntoIterator<Item = Self::I>,
    ) -> impl Iterator<Item = Self::O> {
        inputs.into_iter().flat_map(move |x| self.xap(x))
    }

    // transformations

    type Map<Q, H>
        = M<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        M::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = M<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        M::new(self, FnIns::new(h))
    }

    type Filter<H>
        = FilMap<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        FilMap::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        FilMap::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        FlaMap::new(self, FnFlatMap::new(h))
    }
}

impl<'a, I: 'a + Clone, X: Xap, G: Map<I = X::O, O = &'a I>> XapCloned<'a, I> for M<X, G> {
    type Cloned = M<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy, X: Xap, G: Map<I = X::O, O = &'a I>> XapCopied<'a, I> for M<X, G> {
    type Copied = M<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M::new(self, FnCopied::new())
    }
}
