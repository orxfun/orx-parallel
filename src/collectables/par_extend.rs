use crate::{Par, collectables::ParExtendCore};

/// Extends an existing collection with items produced by a parallel iterator.
pub trait ParExtend<T>: ParExtendCore<T> {
    /// Collects items from `iter` into this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::{IntoParIter, ParExtend};
    ///
    /// let mut values = vec![0];
    /// values.par_extend((1..=4).into_par());
    /// assert_eq!(values, vec![0, 1, 2, 3, 4]);
    ///
    /// use std::collections::HashSet;
    ///
    /// let mut values = HashSet::from([0]);
    /// values.par_extend((1..=4).into_par());
    /// assert_eq!(values, HashSet::from([0, 1, 2, 3, 4]));
    /// ```
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send;
}

impl<T, P: ParExtendCore<T>> ParExtend<T> for P {
    fn par_extend(&mut self, iter: impl Par<Item = T>)
    where
        T: Send,
    {
        iter.collect_into(self);
    }
}
