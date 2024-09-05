use crate::par::par_empty::ParEmpty;
use orx_concurrent_iter::*;

/// Conversion into a parallel iterator.
///
/// Every regular iterator implementing [`Iterator`] also implements `IterIntoPar`.
///
/// See [`crate::IntoPar`] for conversion of common collections into parallel iterator.
///
/// Converting into a parallel iterator is achieved using the `par()` method.
///
/// # Examples
///
/// ```rust
/// use orx_parallel::*;
///
/// let numbers = || (0..24).filter(|x| x % 2 == 0);
/// let seq: usize = numbers().sum();
/// let par = numbers().par().sum();
/// assert_eq!(seq, par);
///
/// let numbers: Vec<_> = (10..420).collect();
/// let iter = || numbers.iter().cloned().skip(10);
/// let seq: Vec<_> = iter()
///     .filter(|x| x % 3 == 1)
///     .map(|x| x * 4)
///     .filter(|x| x < &456)
///     .flat_map(|x| [x, x + 1, x * 7])
///     .collect();
/// let par = iter()
///     .par()
///     .filter(|x| x % 3 == 1)
///     .map(|x| x * 4)
///     .filter(|x| x < &456)
///     .flat_map(|x| [x, x + 1, x * 7])
///     .collect_vec();
/// assert_eq!(par, seq);
/// ```
pub trait IterIntoPar<Iter>
where
    Iter: Iterator,
    Iter::Item: Send + Sync,
{
    /// Underlying concurrent iterator which provides the input elements to the defined parallel computation.
    type ConIter: ConcurrentIter;

    /// Conversion into a parallel iterator.
    ///
    /// Every regular iterator implementing [`Iterator`] also implements `IterIntoPar`.
    ///
    /// See [`crate::IntoPar`] for conversion of common collections into parallel iterator.
    ///
    /// Converting into a parallel iterator is achieved using the `par()` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let numbers = || (0..24).filter(|x| x % 2 == 0);
    /// let seq: usize = numbers().sum();
    /// let par = numbers().par().sum();
    /// assert_eq!(seq, par);
    ///
    /// let numbers: Vec<_> = (10..420).collect();
    /// let iter = || numbers.iter().cloned().skip(10);
    /// let seq: Vec<_> = iter()
    ///     .filter(|x| x % 3 == 1)
    ///     .map(|x| x * 4)
    ///     .filter(|x| x < &456)
    ///     .flat_map(|x| [x, x + 1, x * 7])
    ///     .collect();
    /// let par = iter()
    ///     .par()
    ///     .filter(|x| x % 3 == 1)
    ///     .map(|x| x * 4)
    ///     .filter(|x| x < &456)
    ///     .flat_map(|x| [x, x + 1, x * 7])
    ///     .collect_vec();
    /// assert_eq!(par, seq);
    /// ```
    fn par(self) -> ParEmpty<Self::ConIter>;
}

// iter

impl<Iter> IterIntoPar<Iter> for Iter
where
    Iter: Iterator,
    Iter::Item: Send + Sync,
{
    type ConIter = ConIterOfIter<Iter::Item, Iter>;

    fn par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.into_con_iter())
    }
}

impl<Iter> IterIntoPar<Iter> for ConIterOfIter<Iter::Item, Iter>
where
    Iter: Iterator,
    Iter::Item: Send + Sync,
{
    type ConIter = ConIterOfIter<Iter::Item, Iter>;

    fn par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}
