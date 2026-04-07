use crate::infallible::fun::Map;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Xap};
use crate::result::size_pairs::SizePair;
use core::marker::PhantomData;

pub struct XapRes<M, E, X1, X2, S>
where
    X1: Xap<O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    x1: X1,
    x2: X2,
    s: PhantomData<S>,
}

impl<M, E, X1, X2, S> XapRes<M, E, X1, X2, S>
where
    X1: Xap<O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        let s = PhantomData;
        Self { x1, x2, s }
    }

    #[inline(always)]
    fn xap_res(&self, i: X1::I) -> S::Results<M, E, X1, X2> {
        S::xap_res(self.x1, self.x2, i)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> XapRes<M, E, X1, MapOf<X2, Q, H>, S>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.map(h))
    }

    fn inspect<H>(self, h: H) -> XapRes<M, E, X1, InsOf<X2, H>, S>
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.inspect(h))
    }

    fn filter<H>(self, h: H) -> XapRes<M, E, X1, FilOf<X2, H>, S::ThenBin>
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.filter(h))
    }

    fn filter_map<Q, H>(self, h: H) -> XapRes<M, E, X1, FilMapOf<X2, Q, H>, S::ThenBin>
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.filter_map(h))
    }

    fn flat_map<V, H>(self, h: H) -> XapRes<M, E, X1, FlatMapOf<X2, V, H>, S::ThenMany>
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    fn mapped<H>(self, h: H) -> XapRes<M, E, X1, MappedOf<X2, H>, S>
    where
        H: Map<I = X2::O>,
    {
        XapRes::new(self.x1, self.x2.mapped(h))
    }
}
