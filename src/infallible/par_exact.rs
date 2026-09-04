use crate::{Par, infallible::Xap, sizes::One};
use orx_concurrent_iter::ExactSizeConcurrentIter;

pub trait ExactSizePar: Par {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<P> ExactSizePar for P
where
    P: Par,
    P::Input: ExactSizeConcurrentIter,
    P::Xap: Xap<Size = One>,
{
    fn len(&self) -> usize {
        self.size_hint().0
    }
}
