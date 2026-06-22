use crate::ParCollectInto;
use crate::common_par_traits::{ParInfCommon, ParOptCommon, ParResCommon};

/// Extends a collection from parallel iterators.
///
/// This is the parallel counterpart of [`std::iter::Extend`].
///
/// # Example
///
/// ```
/// use orx_parallel::*;
///
/// let mut out = vec![42];
/// out.par_extend((0..10).par().map(|x| x * 2));
///
/// assert_eq!(out.len(), 11);
/// assert_eq!(out[0], 42);
/// ```
pub trait ParExtend<T>: ParCollectInto<T> {
    /// Extends `self` with items produced by an infallible parallel iterator.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut out = vec![42];
    /// let par = (0..20).par().map(|x| x * 2).filter(|x| *x < 10);
    /// out.par_extend(par);
    ///
    /// assert_eq!(out, vec![42, 0, 2, 4, 6, 8]);
    /// ```
    fn par_extend(&mut self, iter: impl ParInfCommon<CommonItem = T>)
    where
        T: Send,
    {
        iter.common_collect_into(self)
    }

    /// Extends `self` with items produced by an optional parallel iterator.
    ///
    /// Returns `None` if the source computation yields `None`; otherwise returns `Some(())`.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut out = vec![1];
    /// let par = (0..5).par().map(Some).into_optional();
    /// let ok = out.par_extend_opt(par);
    ///
    /// assert_eq!(ok, Some(()));
    /// assert_eq!(out.len(), 6);
    /// ```
    fn par_extend_opt(&mut self, iter: impl ParOptCommon<CommonItem = T>) -> Option<()>
    where
        T: Send,
    {
        iter.common_collect_into(self)
    }

    /// Extends `self` with items produced by a fallible parallel iterator.
    ///
    /// Returns the first error if the source computation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut out = vec![1usize];
    /// let par = (0..5)
    ///     .par()
    ///     .map(Result::<_, char>::Ok)
    ///     .into_fallible();
    ///
    /// let ok = out.par_extend_res(par);
    /// assert_eq!(ok, Ok(()));
    /// assert_eq!(out.len(), 6);
    /// ```
    fn par_extend_res<E>(
        &mut self,
        iter: impl ParResCommon<CommonItem = T, CommonError = E>,
    ) -> Result<(), E>
    where
        T: Send,
        E: Send,
    {
        iter.common_collect_into(self)
    }
}

impl<T, C: ParCollectInto<T>> ParExtend<T> for C {}
