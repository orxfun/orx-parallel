#![allow(clippy::type_complexity)]

use crate::infallible::Xap;
use crate::sizes::SizePair;
use core::iter::Fuse;

/// Sequential iterator for a two-stage result `Xap` pipeline.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct XapResultIter<I, M, E, X1, X2, S>
where
    I: Iterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    iter: Fuse<I>,
    x1: X1,
    x2: X2,
    values: Option<<S::XapResResult<M, E, X1, X2> as IntoIterator>::IntoIter>,
}

impl<I, M, E, X1, X2, S> XapResultIter<I, M, E, X1, X2, S>
where
    I: Iterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    /// Creates a sequential iterator for a result parallel pipeline.
    pub fn new(iter: I, x1: X1, x2: X2) -> Self {
        Self {
            iter: iter.fuse(),
            x1,
            x2,
            values: None,
        }
    }
}

impl<I, M, E, X1, X2, S> Iterator for XapResultIter<I, M, E, X1, X2, S>
where
    I: Iterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(values) = &mut self.values
                && let Some(value) = values.next()
            {
                return Some(value);
            }
            self.values = None;
            let input = self.iter.next()?;
            self.values = Some(S::xap_res(self.x1, self.x2, input).into_iter());
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let current = self
            .values
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let remaining = <S as SizePair>::transformed_size_hint(self.iter.size_hint());
        (
            current.0.saturating_add(remaining.0),
            current
                .1
                .and_then(|a| remaining.1.and_then(|b| a.checked_add(b))),
        )
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut result = init;
        if let Some(values) = self.values {
            result = values.fold(result, &mut f);
        }
        for input in self.iter {
            result = S::xap_res(self.x1, self.x2, input)
                .into_iter()
                .fold(result, &mut f);
        }
        result
    }
}
