use crate::collectables::ParExtendCore;
use crate::{Par, ParOption, ParResult};

/// Extends an existing collection with items produced by a parallel iterator.
pub trait ParExtend<T>: ParExtendCore<T> {
    /// Collects items from `iter` into this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::{IntoParIter, Par, ParExtend};
    /// use std::collections::BTreeSet;
    ///
    /// let mut values = vec![0];
    /// values.par_extend((1..=4).into_par());
    /// assert_eq!(values, vec![0, 1, 2, 3, 4]);
    ///
    /// let mut values = BTreeSet::from([0]);
    /// values.par_extend((1..=4).into_par());
    /// assert_eq!(values, BTreeSet::from([0, 1, 2, 3, 4]));
    /// ```
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send;

    /// Extends this collection with values from an optional parallel iterator.
    ///
    /// Returns `Some(())` when the iterator completes, or `None` when it encounters `None`.
    /// Values produced before the stop condition remain in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::{IntoParIter, Par, ParExtend};
    ///
    /// let mut values = vec![0];
    /// let result = (1..=4).into_par().map(Some).into_optional();
    /// assert_eq!(values.par_extend_optional(result), Some(()));
    /// assert_eq!(values, vec![0, 1, 2, 3, 4]);
    ///
    /// let mut values = vec![0];
    /// let result = (1..=4)
    ///     .into_par()
    ///     .map(|value| (value < 3).then_some(value))
    ///     .into_optional();
    /// assert_eq!(values.par_extend_optional(result), None);
    /// ```
    fn par_extend_optional(&mut self, iter: impl ParOption<Item = T>) -> Option<()>
    where
        T: Send;

    /// Extends this collection with values from a fallible parallel iterator.
    ///
    /// Returns `Ok(())` when the iterator completes, or the first error reported by the iterator.
    /// Values produced before the error remain in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::{IntoParIter, Par, ParExtend};
    ///
    /// let mut values = vec![0];
    /// let result = (1..=4)
    ///     .into_par()
    ///     .map(Result::<_, &str>::Ok)
    ///     .into_fallible();
    /// assert_eq!(values.par_extend_fallible(result), Ok(()));
    /// assert_eq!(values, vec![0, 1, 2, 3, 4]);
    ///
    /// let mut values = vec![0];
    /// let result = (1..=4)
    ///     .into_par()
    ///     .map(|value| if value < 3 { Ok(value) } else { Err("stopped") })
    ///     .into_fallible();
    /// assert_eq!(values.par_extend_fallible(result), Err("stopped"));
    /// ```
    fn par_extend_fallible<I>(&mut self, iter: I) -> Result<(), I::Error>
    where
        I: ParResult<Item = T>,
        I::Error: Send,
        T: Send;
}

impl<T, P: ParExtendCore<T>> ParExtend<T> for P {
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send,
    {
        iter.collect_into(self);
    }

    fn par_extend_optional(&mut self, iter: impl ParOption<Item = T>) -> Option<()>
    where
        T: Send,
    {
        iter.collect_into(self)
    }

    fn par_extend_fallible<I>(&mut self, iter: I) -> Result<(), I::Error>
    where
        I: ParResult<Item = T>,
        I::Error: Send,
        T: Send,
    {
        iter.collect_into(self)
    }
}
