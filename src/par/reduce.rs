use std::{
    cmp::Ordering,
    fmt::Debug,
    ops::{Add, Deref},
};

use crate::ParIter;

/// Trait defining reduction operations.
///
/// This trait requires implementation of the generic `reduce` method.
///
/// Using `reduce`, it provides default implementations various useful reductions such as `sum` or `min`.
pub trait Reduce<T>
where
    Self: Sized,
    T: Send + Sync,
{
    /// Reduces the elements to a single one, by repeatedly applying the `reduce` operation.
    ///
    /// If the iterator is empty, returns None; otherwise, returns the result of the reduction.
    ///
    /// The reducing function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let reduced = (1..10).into_par().reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, Some(45));
    ///
    /// let reduced = (1..10).into_par().filter(|x| *x > 10).reduce(|acc, e| acc + e);
    /// assert_eq!(reduced, None);
    /// ```
    fn reduce<R>(self, reduce: R) -> Option<T>
    where
        R: Fn(T, T) -> T + Send + Sync;

    /// Folds the elements to a single one, by repeatedly applying the `fold` operation starting from the `identity`.
    ///
    /// If the iterator is empty, returns back the `identity`; otherwise, returns the result of the fold.
    ///
    /// The fold function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// Note that, unlike its sequential counterpart, parallel fold requires the `identity` and `fold` to satisfy the following:
    /// * `fold(a, b)` is equal to `fold(b, a)`,
    /// * `fold(a, fold(b, c))` is equal to `fold(fold(a, b), c)`,
    /// * `fold(identity, a)` is equal to `a`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let fold = (1..10).into_par().fold(|| 0, |acc, e| acc + e);
    /// assert_eq!(fold, 45);
    ///
    /// let fold = (1..10).into_par().filter(|x| *x > 10).fold(|| 1, |acc, e| acc * e);
    /// assert_eq!(fold, 1);
    /// ```
    fn fold<Id, F>(self, identity: Id, fold: F) -> T
    where
        Id: Fn() -> T,
        F: Fn(T, T) -> T + Send + Sync,
    {
        self.reduce(fold).unwrap_or_else(identity)
    }

    /// Sums up the items in the iterator.
    ///
    /// Note that the order in items will be reduced is not specified, so if the + operator is not truly associative (as is the case for floating point numbers), then the results are not fully deterministic.
    ///
    /// Basically equivalent to `self.fold(|| 0, |a, b| a + b)`, except that the type of 0 and the + operation may vary depending on the type of value being produced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let sum = (1..10).into_par().sum();
    /// assert_eq!(sum, 45);
    ///
    /// let sum = (1..10).into_par().map(|x| x as f32).sum();
    /// assert!((sum - 45.0).abs() < f32::EPSILON);
    ///
    /// let sum = (1..10).into_par().filter(|x| *x > 10).sum();
    /// assert_eq!(sum, 0);
    /// ```
    fn sum(self) -> T
    where
        T: Default + Add<Output = T>,
    {
        self.reduce(|x, y| x + y).unwrap_or(T::default())
    }

    /// Computes the minimum of all the items in the iterator. If the iterator is empty, None is returned; otherwise, Some(min) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the Ord impl is not truly associative, then the results are not deterministic.
    ///     
    /// Basically equivalent to `self.reduce(|a, b| Ord::min(a, b))`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let min = (1..10).into_par().filter(|x| *x > 6).min();
    /// assert_eq!(min, Some(7));
    ///
    /// let min = (1..10).into_par().filter(|x| *x > 10).min();
    /// assert_eq!(min, None);
    /// ```
    fn min(self) -> Option<T>
    where
        T: Ord,
    {
        self.reduce(|x, y| Ord::min(x, y))
    }

    /// Computes the maximum of all the items in the iterator. If the iterator is empty, None is returned; otherwise, Some(max) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the Ord impl is not truly associative, then the results are not deterministic.
    ///     
    /// Basically equivalent to `self.reduce(|a, b| Ord::max(a, b))`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let max = (1..10).into_par().filter(|x| *x < 6).max();
    /// assert_eq!(max, Some(5));
    ///
    /// let max = (1..10).into_par().filter(|x| *x > 10).max();
    /// assert_eq!(max, None);
    /// ```
    fn max(self) -> Option<T>
    where
        T: Ord,
    {
        self.reduce(|x, y| Ord::max(x, y))
    }

    /// Computes the minimum of all the items in the iterator with respect to the given comparison function.
    /// If the iterator is empty, None is returned; otherwise, Some(min) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the comparison function is not associative, then the results are not deterministic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let names: Vec<_> = ["john", "doe", "adams", "jones", "grumpy"]
    ///     .map(String::from)
    ///     .into_iter()
    ///     .collect();
    ///
    /// let min = names.as_slice().into_par().min_by(|a, b| a.len().cmp(&b.len()));
    /// assert_eq!(min.map(|x| x.as_ref()), Some("doe"));
    ///
    /// let min = names
    ///     .as_slice()
    ///     .into_par()
    ///     .filter(|x| x.starts_with('x'))
    ///     .min_by(|a, b| a.len().cmp(&b.len()));
    /// assert_eq!(min, None);
    /// ```
    fn min_by<F>(self, compare: F) -> Option<T>
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.reduce(|x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        })
    }

    /// Computes the maximum of all the items in the iterator with respect to the given comparison function.
    /// If the iterator is empty, None is returned; otherwise, Some(max) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the comparison function is not associative, then the results are not deterministic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let names: Vec<_> = ["john", "doe", "adams", "jones", "grumpy"]
    ///     .map(String::from)
    ///     .into_iter()
    ///     .collect();
    ///
    /// let max = names.as_slice().into_par().max_by(|a, b| a.len().cmp(&b.len()));
    /// assert_eq!(max.map(|x| x.as_ref()), Some("grumpy"));
    ///
    /// let max = names
    ///     .as_slice()
    ///     .into_par()
    ///     .filter(|x| x.starts_with('x'))
    ///     .max_by(|a, b| a.len().cmp(&b.len()));
    /// assert_eq!(max, None);
    /// ```
    fn max_by<F>(self, compare: F) -> Option<T>
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.reduce(|x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        })
    }

    /// Computes the item that yields the minimum value for the given function. If the iterator is empty, None is returned; otherwise, Some(min) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the Ord impl is not truly associative, then the results are not deterministic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let names: Vec<_> = ["john", "doe", "adams", "jones", "grumpy"]
    ///     .map(String::from)
    ///     .into_iter()
    ///     .collect();
    ///
    /// let min = names.as_slice().into_par().min_by_key(|x| x.len());
    /// assert_eq!(min.map(|x| x.as_ref()), Some("doe"));
    ///
    /// let min = names
    ///     .as_slice()
    ///     .into_par()
    ///     .filter(|x| x.starts_with('x'))
    ///     .min_by_key(|x| x.len());
    /// assert_eq!(min, None);
    /// ```
    fn min_by_key<B, F>(self, get_key: F) -> Option<T>
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        self.reduce(|x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        })
    }

    /// Computes the item that yields the maximum value for the given function. If the iterator is empty, None is returned; otherwise, Some(max) is returned.
    ///
    /// Note that the order in which the items will be reduced is not specified, so if the Ord impl is not truly associative, then the results are not deterministic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let names: Vec<_> = ["john", "doe", "adams", "jones", "grumpy"]
    ///     .map(String::from)
    ///     .into_iter()
    ///     .collect();
    ///
    /// let max = names.as_slice().into_par().max_by_key(|x| x.len());
    /// assert_eq!(max.map(|x| x.as_ref()), Some("grumpy"));
    ///
    /// let max = names
    ///     .as_slice()
    ///     .into_par()
    ///     .filter(|x| x.starts_with('x'))
    ///     .max_by_key(|x| x.len());
    /// assert_eq!(max, None);
    /// ```
    fn max_by_key<B, F>(self, get_key: F) -> Option<T>
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        self.reduce(|x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        })
    }
}
