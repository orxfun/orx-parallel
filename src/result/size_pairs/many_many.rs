use crate::infallible::Xap;
use crate::infallible::sizes::Many;
use crate::result::size_pairs::SizePair;

pub struct ManyMany;

impl SizePair for ManyMany {
    type S1 = Many;

    type S2 = Many;

    type Results<M, E, X1, X2>
        = IterResManyMany<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(&self, x1: X1, x2: X2, i: X1::I) -> Self::Results<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        let (x2, inner) = (x2, None);
        IterResManyMany { iter, x2, inner }
    }
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
