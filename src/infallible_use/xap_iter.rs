use crate::infallible_use::XapUse;
use crate::sizes::Size;
use crate::use_var::Use;
use core::iter::Fuse;

/// Iterator that applies an [`XapUse`] to each input using an owned worker-local
/// value and yields its output values.
///
/// This is equivalent to `iter.flat_map(|i| xap.xap_use(u, i))` for worker
/// index zero while keeping the transformation and use value as a named iterator.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct XapUseIter<U, I, X>
where
    U: Use,
    I: Iterator,
    X: XapUse<U = U::Item, I = I::Item>,
{
    _using: U,
    iter: Fuse<I>,
    xap: X,
    values: Option<<X::Values as IntoIterator>::IntoIter>,
    use_ptr: *mut U::Item,
}

impl<U, I, X> XapUseIter<U, I, X>
where
    U: Use,
    I: Iterator,
    X: XapUse<U = U::Item, I = I::Item>,
{
    /// Creates an iterator over the values produced by `xap` for each item in `iter`.
    pub fn new(using: U, iter: I, xap: X) -> Self {
        // SAFETY: we hold on to `using` together with the pointer throughout the life of this iter
        let use_ptr = unsafe { using.init_get(0) } as *mut U::Item;
        Self {
            _using: using,
            iter: iter.fuse(),
            xap,
            values: None,
            use_ptr,
        }
    }
}

impl<U, I, X> Iterator for XapUseIter<U, I, X>
where
    U: Use,
    I: Iterator,
    X: XapUse<U = U::Item, I = I::Item>,
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
            self.values = Some(self.xap.xap_use(self.use_ptr, input).into_iter());
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
            <X::Size as Size>::transformed_size_hint((iter_lower, iter_upper));

        let lower = values_lower.saturating_add(transformed_lower);
        let upper = values_upper.and_then(|values_upper| {
            transformed_upper
                .and_then(|transformed_upper| values_upper.checked_add(transformed_upper))
        });
        (lower, upper)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut result = init;
        if let Some(values) = self.values.take() {
            result = values.fold(result, &mut f);
        }
        for input in self.iter.by_ref() {
            result = self
                .xap
                .xap_use(self.use_ptr, input)
                .into_iter()
                .fold(result, &mut f);
        }
        result
    }
}
