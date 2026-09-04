#![allow(clippy::type_complexity)]

use crate::infallible_use::XapUse;
use crate::sizes::SizePair;
use crate::use_var::Use;
use core::iter::Fuse;

/// Sequential iterator for a two-stage result `XapUse` pipeline.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct XapUseResultIter<U, I, M, E, X1, X2, S>
where
    U: Use,
    I: Iterator,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    _using: U,
    iter: Fuse<I>,
    x1: X1,
    x2: X2,
    values: Option<<S::XapUseResResult<M, E, X1, X2> as IntoIterator>::IntoIter>,
    use_ptr: *mut U::Item,
}

impl<U, I, M, E, X1, X2, S> XapUseResultIter<U, I, M, E, X1, X2, S>
where
    U: Use,
    I: Iterator,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    /// Creates a sequential iterator for a result parallel pipeline.
    pub fn new(using: U, iter: I, x1: X1, x2: X2) -> Self {
        // SAFETY: the pointer is kept with `using` for the iterator's lifetime.
        let use_ptr = unsafe { using.init_get(0) } as *mut U::Item;
        Self {
            _using: using,
            iter: iter.fuse(),
            x1,
            x2,
            values: None,
            use_ptr,
        }
    }
}

impl<U, I, M, E, X1, X2, S> Iterator for XapUseResultIter<U, I, M, E, X1, X2, S>
where
    U: Use,
    I: Iterator,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
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
            self.values = Some(S::xap_use_res(self.use_ptr, self.x1, self.x2, input).into_iter());
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
            result = S::xap_use_res(self.use_ptr, self.x1, self.x2, input)
                .into_iter()
                .fold(result, &mut f);
        }
        result
    }
}
