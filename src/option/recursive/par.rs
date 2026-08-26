#![allow(clippy::type_complexity)]

use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::option::recursive::par_core::ParRecOptionCore;
use crate::runner::ParRunner;
use crate::{ChunkSize, IterationOrder, NumThreads};
use crate::{ParCollectInto, Sum};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Fallible recursive parallel iterator over `Option` values.
///
/// `ParRecOption` is the recursive counterpart of [`ParOption`](crate::ParOption): each
/// visited node may fail as `None`, in which case the whole computation short-circuits.
/// It is created from [`ParRec`](crate::ParRec) with
/// [`into_optional`](crate::ParRec::into_optional). When a node maps to `None`, its
/// children are **not** discovered/visited.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let children: Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5], vec![], vec![], vec![]];
///
/// let mut ok: Option<Vec<usize>> = [0usize]
///     .into_par_rec(|node| children[*node].iter().copied())
///     .map(|x| (x <= 5).then_some(x))
///     .into_optional()
///     .map(|x| x * 2)
///     .collect();
/// ok.as_mut().unwrap().sort();
/// assert_eq!(ok, Some(vec![0, 2, 4, 6, 8, 10]));
///
/// let fail: Option<Vec<usize>> = [0usize]
///     .into_par_rec(|node| children[*node].iter().copied())
///     .map(|x| (x != 5).then_some(x))
///     .into_optional()
///     .map(|x| x * 2)
///     .collect();
/// assert_eq!(fail, None);
/// ```
pub trait ParRecOption: Sized + ParRecOptionCore {
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let children: Vec<Vec<i32>> = vec![vec![1, 2], vec![], vec![]];
    ///
    /// let par = [0i32]
    ///     .into_par_rec(|node| children[*node as usize].iter().copied())
    ///     .map(Some)
    ///     .into_optional();
    ///
    /// let par = par.runner(Runner::fixed());
    ///
    /// let mut out: Option<Vec<_>> = par.collect();
    /// out.as_mut().unwrap().sort();
    /// assert_eq!(out, Some(vec![0, 1, 2]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParRecOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    >;

    /// Wraps the current parallel runner with a diagnostics-enabled runner.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "std")]
    /// # fn main() {
    /// use orx_parallel::*;
    ///
    /// let par = [1i32]
    ///     .into_par_rec(|&x| (x < 100).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .num_threads(4);
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let sum: Option<i32> = par.sum();
    /// assert_eq!(sum, Some(5050));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParRecOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    >;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: Option<i32> = [1i32]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .num_threads(2)
    ///     .sum();
    /// assert_eq!(sum, Some(15));
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values: Option<Vec<_>> = [0usize]
    ///     .into_par_rec(|&x| (x < 31).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .chunk_size(8)
    ///     .map(|x| x + 1)
    ///     .collect();
    ///
    /// let values = values.as_mut().unwrap();
    /// values.sort();
    /// assert_eq!(values.len(), 32);
    /// assert_eq!(values[0], 1);
    /// assert_eq!(values[31], 32);
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration order semantics for operations sensitive to ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = [1i32]
    ///     .into_par_rec(|&x| (x < 9_999).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(Some(3421)));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    /// Maps each successful element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .map(|x| 2 * x)
    ///     .collect();
    /// assert_eq!(out, Some(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecOption<
        Item = Q,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each successful element and forwards the element unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let seen = AtomicUsize::new(0);
    /// let out: Option<Vec<_>> = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .inspect(|_| {
    ///         seen.fetch_add(1, Ordering::Relaxed);
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(out.map(|mut v| { v.sort(); v }), Some(vec![1, 2, 3, 4]));
    /// assert_eq!(seen.load(Ordering::Relaxed), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParRecOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&Self::Item) + Copy + Send;

    /// Keeps successful elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = [1i32]
    ///     .into_par_rec(|&x| (x < 6).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .filter(|x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParRecOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    /// Maps and filters successful elements in a single pass.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = ["1", "x", "5"]
    ///     .into_par_rec(|_: &&str| None::<&str>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecOption<
        Item = Q,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each successful element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .flat_map(|x| [x, x + 10])
    ///     .collect();
    /// assert_eq!(out, Some(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParRecOption<
        Item = V::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let mut out: Option<Vec<_>> = nested
    ///     .into_par_rec(|_: &Vec<i32>| None::<Vec<i32>>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .flatten()
    ///     .collect();
    /// out.as_mut().unwrap().sort();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParRecOption<
        Item = <Self::Item as IntoIterator>::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first successful item, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let first = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .first();
    /// assert_eq!(first, Some(Some(1)));
    /// ```
    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Reduces successful items into one value using `f`, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let reduced = [1i32]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .reduce(|a, b| a + b);
    /// assert_eq!(reduced, Some(Some(15)));
    /// ```
    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Collects successful items into `dst`, or returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10];
    /// let ok = [0i32]
    ///     .into_par_rec(|&x| (x < 2).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .collect_into(&mut dst);
    /// assert_eq!(ok, Some(()));
    /// dst.sort();
    /// assert_eq!(dst, vec![0, 1, 2, 10]);
    /// ```
    fn collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Collects successful items into a new collection, or returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = [1i32]
    ///     .into_par_rec(|&x| (x < 3).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .map(|x| x * 2)
    ///     .collect();
    /// assert_eq!(out, Some(vec![2, 4, 6]));
    /// ```
    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    // compute - derived

    /// Returns `Some(true)` if all successful items satisfy `f`; `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .all(|x| x > &0);
    /// assert_eq!(out, Some(true));
    /// ```
    fn all<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|x| f(&x)).find(|x| !*x).map(|x| x.is_none())
    }

    /// Returns `Some(true)` if any successful item satisfies `f`; `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .any(|x| x % 2 == 0);
    /// assert_eq!(out, Some(true));
    /// ```
    fn any<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|x| f(&x)).find(|x| *x).map(|x| x.is_some())
    }

    /// Counts successful elements, or returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = [1i32]
    ///     .into_par_rec(|&x| (x < 10).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .filter(|x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(n, Some(3));
    /// ```
    fn count(self) -> Option<usize>
    where
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
    }

    /// Finds first successful item satisfying `f`, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = [1i32]
    ///     .into_par_rec(|&x| (x < 100).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .find(|x| x % 17 == 0);
    /// assert_eq!(found, Some(Some(17)));
    /// ```
    fn find<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.filter(&f).first()
    }

    /// Folds successful elements into per-thread accumulators, or returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let partials = [1usize]
    ///     .into_par_rec(|&x| (x < 5).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .fold(|| 0usize, |acc, x| *acc += x);
    /// assert_eq!(partials.as_ref().map(|v| v.iter().sum::<usize>()), Some(15));
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Option<Vec<B>>
    where
        B: Send + Sync,
        I: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone;

    /// Executes `f` for each successful element, or returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    /// use orx_parallel::*;
    ///
    /// let total = AtomicUsize::new(0);
    /// let ok = [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .for_each(|x| {
    ///         total.fetch_add(x, Ordering::Relaxed);
    ///     });
    ///
    /// assert_eq!(ok, Some(()));
    /// assert_eq!(total.load(Ordering::Relaxed), 10);
    /// ```
    fn for_each<F>(self, f: F) -> Option<()>
    where
        F: Fn(Self::Item) + Send + Copy,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }

    /// Returns maximum successful element, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let max = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .max();
    /// assert_eq!(max, Some(Some(4)));
    /// ```
    fn max(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.reduce(Ord::max)
    }

    /// Returns successful element considered maximum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .max_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with maximum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .max_by_key(|x| x.abs());
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns minimum successful element, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let min = [1i32]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .min();
    /// assert_eq!(min, Some(Some(1)));
    /// ```
    fn min(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.reduce(Ord::min)
    }

    /// Returns successful element considered minimum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .min_by(|a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with minimum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par_rec(|_: &i32| None::<i32>)
    ///     .map(Some)
    ///     .into_optional()
    ///     .min_by_key(|x| x.abs());
    /// assert_eq!(x, Some(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums successful elements using the [`Sum`] implementation, or `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: Option<usize> = [1usize]
    ///     .into_par_rec(|&x| (x < 4).then_some(x + 1))
    ///     .map(Some)
    ///     .into_optional()
    ///     .sum();
    /// assert_eq!(sum, Some(10));
    /// ```
    fn sum<S>(self) -> Option<S>
    where
        Self::Item: Sum<S>,
        S: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
