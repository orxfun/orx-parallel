use crate::ParIter;
use std::fmt::Debug;

/// Transforms a parallel iterator yielding &T into one that yields T by cloning each element.
///
/// Transformation is via the `cloned` method.
///
/// # Examples
/// ```rust
/// use orx_parallel::*;
///
/// fn warn(mut name: String) -> String {
///     name.push('!');
///     name
/// }
///
/// let names = vec![String::from("john"), String::from("doe")];
///
/// let new_names = names.par().cloned().map(warn).collect_vec();
///
/// assert_eq!(new_names, &[String::from("john!"), String::from("doe!")]);
/// ```
pub trait ParIntoCloned<'a, T>: ParIter<Item = &'a T>
where
    T: Send + Sync + Debug + Clone + 'a,
{
    /// Transforms a parallel iterator yielding &T into one that yields T by cloning each element.
    ///
    /// Transformation is via the `cloned` method.
    ///
    /// # Examples
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// fn warn(mut name: String) -> String {
    ///     name.push('!');
    ///     name
    /// }
    ///
    /// let names = vec![String::from("john"), String::from("doe")];
    ///
    /// let new_names = names.par().cloned().map(warn).collect_vec();
    ///
    /// assert_eq!(new_names, &[String::from("john!"), String::from("doe!")]);
    /// ```
    fn cloned(self) -> impl ParIter<Item = T> {
        self.map(|x| x.clone())
    }
}

impl<'a, T, P> ParIntoCloned<'a, T> for P
where
    T: Send + Sync + Debug + Clone + 'a,
    P: ParIter<Item = &'a T>,
{
}

/// Transforms a parallel iterator yielding &T into one that yields T by copying each element.
///
/// Transformation is via the `copied` method.
///
/// # Examples
/// ```rust
/// use orx_parallel::*;
///
/// let numbers = vec![1, 2, 3, 4];
///
/// let sum = numbers.par().copied().sum();
/// let product = numbers.par().copied().fold(|| 1, |x, y| x * y);
///
/// assert_eq!(sum, 10);
/// assert_eq!(product, 24);
/// ```
pub trait ParIntoCopied<'a, T>: ParIter<Item = &'a T>
where
    T: Send + Sync + Debug + Copy + 'a,
{
    /// Transforms a parallel iterator yielding &T into one that yields T by copying each element.
    ///
    /// Transformation is via the `copied` method.
    ///
    /// # Examples
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let numbers = vec![1, 2, 3, 4];
    ///
    /// let sum = numbers.par().copied().sum();
    /// let product = numbers.par().copied().fold(|| 1, |x, y| x * y);
    ///
    /// assert_eq!(sum, 10);
    /// assert_eq!(product, 24);
    /// ```
    fn copied(self) -> impl ParIter<Item = T> {
        self.map(|x| *x)
    }
}

impl<'a, T, P> ParIntoCopied<'a, T> for P
where
    T: Send + Sync + Debug + Copy + 'a,
    P: ParIter<Item = &'a T>,
{
}
