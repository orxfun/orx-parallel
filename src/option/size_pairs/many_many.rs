use crate::infallible::Xap;
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::{Many, ManyMany};
use core::iter::FusedIterator;

impl SizePairOpt for ManyMany {
    type XapOptResult<M, X1, X2>
        = IterOptManyMany<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        let (x2, inner) = (x2, None);
        IterOptManyMany { iter, x2, inner }
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
                return elt.map(Some);
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

impl<M, I, X2> FusedIterator for IterOptManyMany<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
    X2::Values: FusedIterator,
{
}
