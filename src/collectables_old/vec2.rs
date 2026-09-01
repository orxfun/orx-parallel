use alloc::vec;
use alloc::vec::Vec;

/// A dedicated 2D collection target for parallel `.collect()` operations.
///
/// This type helps make collection intent explicit and avoids ambiguous type inference.
/// In practice:
/// - Use `Vec<_>` for 1D collection results.
/// - Use `Vec2<_>` for 2D collection results.
///
/// # Example
///
/// ```ignore
/// use orx_parallel::*;
///
/// let one_dim: Vec<_> = (0..100).into_par().collect();
/// let two_dim: Vec2<_> = (0..100)
///     .into_par()
///     .map(|x| [x, x + 1])
///     .collect();
///
/// // you may get underlying 2D vec with `into_inner`
/// let two_dim: Vec<Vec<_>> = two_dim.into_inner();
/// ```
#[derive(Debug)]
pub struct Vec2<T> {
    pub(super) inner: Vec<Vec<T>>,
}

impl<T> Vec2<T> {
    /// Consumes `Vec2` and returns the underlying `Vec<Vec<T>>`.
    pub fn into_inner(self) -> Vec<Vec<T>> {
        self.inner
    }
}

impl<T> From<Vec<Vec<T>>> for Vec2<T> {
    fn from(inner: Vec<Vec<T>>) -> Self {
        Self { inner }
    }
}

impl<T> From<Vec2<T>> for Vec<Vec<T>> {
    fn from(value: Vec2<T>) -> Self {
        value.inner
    }
}

impl<T> FromIterator<T> for Vec2<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: vec![iter.into_iter().collect()],
        }
    }
}

impl<T> Extend<T> for Vec2<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.push(iter.into_iter().collect());
    }
}

impl<T> IntoIterator for Vec2<T> {
    type Item = T;
    type IntoIter = core::iter::Flatten<alloc::vec::IntoIter<Vec<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().flatten()
    }
}
