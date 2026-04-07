use crate::infallible::fun::Map;
use crate::infallible::sizes::Many;
use crate::infallible::{MapOf, Xap};
use crate::result::xap_res::{InOf, XapRes};

pub struct XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
}

impl<M, E, X1, X2> XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Size = Many;

    type Results = IterResManyMany<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: InOf<Self>) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        let (x2, inner) = (self.x2, None);
        IterResManyMany { iter, x2, inner }
    }

    // // transformations

    // type Map<Q, H>
    //     = XapResManyMany<M, E, X1, MapOf<X2, Q, H>>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send;

    // fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send,
    // {
    //     XapResManyMany::new(self.x1, self.x2.map(h))
    // }

    // type Inspect<H>
    //     = XapResManyMany<M, E, X1, X2::Inspect<H>>
    // where
    //     H: Fn(&Self::O) + Copy + Send;

    // fn inspect<H>(self, h: H) -> Self::Inspect<H>
    // where
    //     H: Fn(&Self::O) + Copy + Send,
    // {
    //     XapResManyMany::new(self.x1, self.x2.inspect(h))
    // }

    // type Filter<H>
    //     = XapResManyMany<M, E, X1, X2::Filter<H>>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send;

    // fn filter<H>(self, h: H) -> Self::Filter<H>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send,
    // {
    //     XapResManyMany::new(self.x1, self.x2.filter(h))
    // }

    // type FilterMap<Q, H>
    //     = XapResManyMany<M, E, X1, X2::FilterMap<Q, H>>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send;

    // fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send,
    // {
    //     XapResManyMany::new(self.x1, self.x2.filter_map(h))
    // }

    // type FlatMap<V, H>
    //     = XapResManyMany<M, E, X1, X2::FlatMap<V, H>>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send;

    // fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send,
    // {
    //     XapResManyMany::new(self.x1, self.x2.flat_map(h))
    // }

    // // transformations - helper

    // type Mapped<H>
    //     = XapResManyMany<M, E, X1, X2::Mapped<H>>
    // where
    //     H: Map<I = Self::O>;

    // fn mapped<H>(self, h: H) -> Self::Mapped<H>
    // where
    //     H: Map<I = Self::O>,
    // {
    //     XapResManyMany::new(self.x1, self.x2.mapped(h))
    // }
}

// iter

pub struct IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Many>,
{
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, E, I, X2> Iterator for IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Many>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt.map(Ok);
            }

            match self.iter.next() {
                Some(Ok(i)) => self.inner = Some(self.x2.xap(i).into_iter()),
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}

#[inline(always)]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}
