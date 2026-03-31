use crate::infallible::fun::Map;
use crate::infallible::size::{Many, One};
use crate::infallible::{Xap, XapOne};
use crate::option::xap_opt::XapOpt;

pub struct XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, X1, X2> Clone for XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, X1, X2> Copy for XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
}

unsafe impl<M, X1, X2> Send for XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
}

impl<M, X1, X2> XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, X1, X2> XapOpt for XapOptOneMany<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = One>,
    X2: Xap<I = M, Size = Many>,
{
    type I = X1::I;

    type M = M;

    type O = X2::O;

    type Results = IterOptOneMany<<<X2 as Xap>::Values as IntoIterator>::IntoIter>;

    #[inline]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        match self.x1.one_value(i) {
            Some(a) => IterOptOneMany::ok(self.x2.xap(a).into_iter()),
            None => IterOptOneMany::err(),
        }
    }

    // transformations

    type Map<Q, H>
        = XapOptOneMany<M, X1, X2::Map<Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapOptOneMany::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapOptOneMany<M, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapOptOneMany::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapOptOneMany<M, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapOptOneMany::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapOptOneMany<M, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapOptOneMany::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapOptOneMany<M, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapOptOneMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapOptOneMany<M, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapOptOneMany::new(self.x1, self.x2.mapped(h))
    }
}

// iter

pub enum IterOptOneMany<I: Iterator> {
    Ok(I),
    Err(bool),
}

impl<I: Iterator> IterOptOneMany<I> {
    pub fn ok(i: I) -> Self {
        Self::Ok(i)
    }

    pub fn err() -> Self {
        Self::Err(false)
    }
}

impl<I: Iterator> Iterator for IterOptOneMany<I> {
    type Item = Option<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ok(iter) => iter.next().map(Some),
            Self::Err(taken) => match taken {
                false => {
                    // SAFETY: error can be taken out only once
                    *taken = true;
                    Some(None)
                }
                true => None, // the error is already returned
            },
        }
    }
}
