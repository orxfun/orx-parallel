use crate::infallible::fun::Map;
use crate::infallible::sizes::{Bin, Many};
use crate::infallible::{MapOf, Xap, XapBin};
use crate::result::xap_res::{InOf, XapRes};

pub struct XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
}

impl<M, E, X1, X2> XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Size = Many;

    type Results = IterResBinMany<<<X2 as Xap>::Values as IntoIterator>::IntoIter, E>;

    fn xap_res(&self, i: InOf<Self>) -> Self::Results {
        match self.x1.bin_value(i) {
            Some(Ok(a)) => IterResBinMany::success(Some(self.x2.xap(a).into_iter())),
            Some(Err(e)) => IterResBinMany::fail(e),
            None => IterResBinMany::success(None),
        }
    }

    // // transformations

    // type Map<Q, H>
    //     = XapResBinMany<M, E, X1, MapOf<X2, Q, H>>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send;

    // fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.map(h))
    // }

    // type Inspect<H>
    //     = XapResBinMany<M, E, X1, X2::Inspect<H>>
    // where
    //     H: Fn(&Self::O) + Copy + Send;

    // fn inspect<H>(self, h: H) -> Self::Inspect<H>
    // where
    //     H: Fn(&Self::O) + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.inspect(h))
    // }

    // type Filter<H>
    //     = XapResBinMany<M, E, X1, X2::Filter<H>>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send;

    // fn filter<H>(self, h: H) -> Self::Filter<H>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.filter(h))
    // }

    // type FilterMap<Q, H>
    //     = XapResBinMany<M, E, X1, X2::FilterMap<Q, H>>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send;

    // fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.filter_map(h))
    // }

    // type FlatMap<V, H>
    //     = XapResBinMany<M, E, X1, X2::FlatMap<V, H>>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send;

    // fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.flat_map(h))
    // }

    // // transformations - helper

    // type Mapped<H>
    //     = XapResBinMany<M, E, X1, X2::Mapped<H>>
    // where
    //     H: Map<I = Self::O>;

    // fn mapped<H>(self, h: H) -> Self::Mapped<H>
    // where
    //     H: Map<I = Self::O>,
    // {
    //     XapResBinMany::new(self.x1, self.x2.mapped(h))
    // }
}

// iter

pub enum IterResBinMany<I: Iterator, E> {
    Success(Option<I>),
    Fail(Option<E>),
}

impl<I: Iterator, E> IterResBinMany<I, E> {
    pub fn success(i: Option<I>) -> Self {
        Self::Success(i)
    }

    pub fn fail(e: E) -> Self {
        Self::Fail(Some(e))
    }
}

impl<I: Iterator, E> Iterator for IterResBinMany<I, E> {
    type Item = Result<I::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(Some(iter)) => iter.next().map(Ok),
            Self::Success(None) => None,
            Self::Fail(e) => match e.is_some() {
                true => {
                    // SAFETY: error can be taken out only once; and on construction
                    // the error variant must be created with Some of an error
                    Some(Err(unsafe { e.take().unwrap_unchecked() }))
                }
                false => None, // the error is already taken and returned
            },
        }
    }
}
