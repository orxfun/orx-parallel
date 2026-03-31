use crate::infallible::{Many, Xap};
use crate::result::xap_res::XapRes;

pub struct XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = Many>,
    X2: Xap<I = M, Count = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = Many>,
    X2: Xap<I = M, Count = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResManyMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = Many>,
    X2: Xap<I = M, Count = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = IterResManyMany<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: Self::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        let (x2, inner) = (self.x2, None);
        IterResManyMany { iter, x2, inner }
    }
}

// iter

pub struct IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Count = Many>,
{
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, E, I, X2> Iterator for IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Count = Many>,
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
