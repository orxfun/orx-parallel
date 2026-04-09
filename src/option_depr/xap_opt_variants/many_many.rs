use crate::infallible::fun::Map;
use crate::infallible::sizes::Many;
use crate::infallible::{MapOf, Xap};
use crate::option::xap_opt::XapOpt;

pub struct XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, X1, X2> Clone for XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, X1, X2> Copy for XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
}

unsafe impl<M, X1, X2> Send for XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
}

impl<M, X1, X2> XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, X1, X2> XapOpt for XapOptManyMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Many>,
    X2: Xap<I = M, Size = Many>,
{
    type I = X1::I;

    type M = M;

    type O = X2::O;

    type Results = IterOptManyMany<M, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: Self::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        let (x2, inner) = (self.x2, None);
        IterOptManyMany { iter, x2, inner }
    }

    // transformations

    type Map<Q, H>
        = XapOptManyMany<M, X1, MapOf<X2, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapOptManyMany::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapOptManyMany<M, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapOptManyMany::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapOptManyMany<M, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapOptManyMany::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapOptManyMany<M, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapOptManyMany::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapOptManyMany<M, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapOptManyMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapOptManyMany<M, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapOptManyMany::new(self.x1, self.x2.mapped(h))
    }
}

// iter

pub struct IterOptManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
{
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, I, X2> Iterator for IterOptManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return Some(elt);
            }

            match self.iter.next() {
                Some(Some(i)) => self.inner = Some(self.x2.xap(i).into_iter()),
                Some(None) => return Some(None),
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
