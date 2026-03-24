use crate::xap::count::Count;
use crate::xap::fun::filter_map::{FnFil, FnFilMap};
use crate::xap::fun::flat_map::{FlatMap, FnFlatMap};
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap};
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};

pub struct FlaMap<X: Xap, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FlatMap<I = X::O>> Clone for FlaMap<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap, G: FlatMap<I = X::O>> Copy for FlaMap<X, G> {}

impl<X: Xap, G: FlatMap<I = X::O>> FlaMap<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FlatMap<I = X::O>> Xap for FlaMap<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Count = <X::Count as Count>::ThenMany;

    type Values = <X::Count as Count>::FlatMap<X::Values, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        <X::Count as Count>::flat_map(self.x.xap(i), self.g)
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

impl<'a, I: 'a + Clone, X: Xap, G: FlatMap<I = X::O>> XapCloned<'a, I> for FlaMap<X, G>
where
    G::O: IntoIterator<Item = &'a I>,
{
    type Cloned = M<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy, X: Xap, G: FlatMap<I = X::O>> XapCopied<'a, I> for FlaMap<X, G>
where
    G::O: IntoIterator<Item = &'a I>,
{
    type Copied = M<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M::new(self, FnCopied::new())
    }
}
