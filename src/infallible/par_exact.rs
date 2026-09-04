/// Parallel iterator with an exact number of output items.///
/// This trait is implemented when the input is exact-sized and the pipeline
/// applies a one-to-one transformation.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let values = (0..4).into_par();
/// assert_eq!(values.len(), 4);
/// assert!(!values.is_empty());
/// ```
pub trait ExactSizePar {
    /// Returns the exact number of output items.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((0..10).into_par().len(), 10);
    /// assert_eq!((0..10).into_par().map(|x| x + 2).len(), 10);
    /// ```
    fn len(&self) -> usize;

    /// Returns `true` when the parallel iterator has no output items.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert!((0..0).into_par().is_empty());
    /// assert!(!(0..1).into_par().is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
