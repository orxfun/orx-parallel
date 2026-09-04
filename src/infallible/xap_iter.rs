use crate::infallible::Xap;
use core::iter::Fuse;

/// Iterator that applies an [`Xap`] to each input and yields its output values.
///
/// This is equivalent to `iter.flat_map(|i| xap.xap(i))` while keeping the
/// transformation as a named iterator.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct XapIter<I, X>
where
    I: Iterator,
    X: Xap<I = I::Item>,
{
    iter: Fuse<I>,
    xap: X,
    values: Option<<X::Values as IntoIterator>::IntoIter>,
}

impl<I, X> XapIter<I, X>
where
    I: Iterator,
    X: Xap<I = I::Item>,
{
    /// Creates an iterator over the values produced by `xap` for each item in `iter`.
    pub fn new(iter: I, xap: X) -> Self {
        Self {
            iter: iter.fuse(),
            xap,
            values: None,
        }
    }
}

impl<I, X> Iterator for XapIter<I, X>
where
    I: Iterator,
    X: Xap<I = I::Item>,
{
    type Item = X::O;

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
            self.values = Some(self.xap.xap(input).into_iter());
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (values_lower, values_upper) = self
            .values
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let (iter_lower, iter_upper) = self.iter.size_hint();
        let (transformed_lower, transformed_upper) =
            <X::Size as crate::sizes::Size>::transformed_size_hint((iter_lower, iter_upper));

        let lower = values_lower.saturating_add(transformed_lower);
        let upper = values_upper.and_then(|values_upper| {
            transformed_upper
                .and_then(|transformed_upper| values_upper.checked_add(transformed_upper))
        });
        (lower, upper)
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
            result = self.xap.xap(input).into_iter().fold(result, &mut f);
        }
        result
    }
}
