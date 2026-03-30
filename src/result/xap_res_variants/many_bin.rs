use crate::infallible::size::{Bin, Many};
use crate::infallible::xap::{Xap, XapBin};
use crate::result::xap_res::XapRes;

pub struct XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = IterResManyBin<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        IterResManyBin { iter, x2: self.x2 }
    }

    // transformations

    type Map<Q, H>
        = XapResManyBin<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResManyBin<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.inspect(h))
    }
}

// iter

pub struct IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(a)) => {
                    let b = self.x2.bin_value(a);
                    if b.is_some() {
                        return b.map(Ok);
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}
