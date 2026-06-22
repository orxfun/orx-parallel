#![allow(clippy::type_complexity)]

use core::cmp::Ordering;

use crate::common_par_traits::ParOptCommon;
use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, XapUse,
};
use crate::option_use::ParUseOptionIter;
use crate::option_use::par_core::ParUseOptionCore;
use crate::pool::ParThreadPool;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Sum};

/// Fallible parallel iterator with worker-local mutable state.
///
/// `ParUseOption` combines:
/// - fallible processing (`Option`-based short-circuiting), and
/// - mutable worker-local state (`&mut Use` passed to closures).
///
/// You can enter this mode from [`ParOption`](crate::ParOption) via
/// [`use_new`](crate::ParOption::use_new),
/// [`use_vec`](crate::ParOption::use_vec), or
/// [`use_slice`](crate::ParOption::use_slice).
///
/// Similar to using `?`, this trait keeps pipeline logic focused on successful
/// values while the computation short-circuits to `None` when any element
/// evaluates to `None`.
///
/// Related traits:
/// - [`ParOption`](crate::ParOption) for `Option`-fallible pipelines without worker-local state,
/// - [`ParUse`](crate::ParUse) for worker-local state in infallible pipelines.
///
/// # Examples
///
/// Reusing a per-worker buffer:
///
/// ```
/// use core::fmt::Write;
/// use orx_parallel::*;
///
/// let out: Option<Vec<usize>> = (0..16usize)
///     .into_par()
///     .map(Some)
///     .into_optional()
///     .use_new(|_| String::with_capacity(32))
///     .map(|buffer, x| {
///         buffer.clear();
///         write!(buffer, "{x}").unwrap();
///         buffer.parse::<usize>().unwrap()
///     })
///     .collect();
///
/// assert_eq!(out, Some((0..16).collect::<Vec<_>>()));
/// ```
///
/// Using RNG as mutable worker-local state:
///
/// ```
/// use orx_parallel::*;
/// use rand::prelude::*;
/// use rand_chacha::ChaCha8Rng;
///
/// let out: Option<Vec<usize>> = (0..32usize)
///     .into_par()
///     .map(Some)
///     .into_optional()
///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(100 + thread_idx as u64))
///     .map(|rng, x| x + rng.random_range(0..10))
///     .collect();
///
/// assert_eq!(out.as_ref().map(Vec::len), Some(32));
/// assert!(out.unwrap().into_iter().enumerate().all(|(i, v)| v >= i));
/// ```
pub trait ParUseOption: Sized + ParUseOptionCore + ParOptCommon<CommonItem = Self::Item> {
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![cfg(feature = "std")]
    ///
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .runner(Runner::fixed_chunk(Pool::once(4)))
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    #[cfg(feature = "std")]
    /// Wraps the current runner with diagnostics-enabled execution.
    ///
    /// # Examples
    ///
    /// ```
    /// #![cfg(feature = "std")]
    ///
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .map(|s| s.parse::<usize>().ok())
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .runner_with_diagnostics()
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    /// Replaces the pool used by the current runner.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// #[cfg(feature = "std")]
    /// {
    ///     let out: Option<Vec<_>> = ["1", "2", "3"]
    ///         .into_par()
    ///         .map(|s| s.parse::<usize>().ok())
    ///         .into_optional()
    ///         .use_new(|_| ())
    ///         .pool(Pool::once(4))
    ///         .collect();
    ///     assert_eq!(out, Some(vec![1, 2, 3]));
    /// }
    /// ```
    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..6)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .num_threads(1)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4, 5]));
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (0..8)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .chunk_size(2)
    ///     .collect();
    ///
    /// assert_eq!(out, Some((0..8).collect::<Vec<_>>()));
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration-order semantics for order-sensitive operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = (1..10_000)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|_, x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(Some(3421)));
    /// ```
    fn iteration_order(self, iteration_order: IterationOrder) -> Self;

    // kind transformations

    /// Copies elements of a reference iterator on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![1, 2, 3];
    /// let out: Option<Vec<_>> = data
    ///     .par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .copied()
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl ParUseOption<
        Item = O,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCopied<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParUseOption<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOptionIter::new(u, iter, x1, x2.mapped(UFnCopied::new()), exe, params)
    }

    /// Clones elements of a reference iterator on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec!["a".to_string(), "b".to_string()];
    /// let out: Option<Vec<_>> = data
    ///     .par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .cloned()
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec!["a".to_string(), "b".to_string()]));
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl ParUseOption<
        Item = O,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCloned<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParUseOption<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOptionIter::new(u, iter, x1, x2.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    /// Maps each successful element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..4)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .map(|_, x| 2 * x)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![2, 4, 6]));
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Q,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each successful element and forwards the item unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut use_vec = UseVec::new(|_| 0usize);
    /// let out: Option<Vec<_>> = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_vec(&mut use_vec)
    ///     .inspect(|count, _| *count += 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
    /// assert_eq!(use_vec.into_vec().into_iter().sum::<usize>(), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send;

    /// Keeps successful elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..7)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 3, 5]));
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send;

    /// Maps and filters successful elements in one pass.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = ["1", "x", "5"]
    ///     .into_par()
    ///     .map(|s| Some(s))
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .filter_map(|_, s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 5]));
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Q,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each successful element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..4)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .flat_map(|_, x| [x, x + 10])
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 11, 2, 12, 3, 13]));
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = V::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        V: IntoIterator,
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables on the success path.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let out: Option<Vec<_>> = nested
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3, 4]));
    /// ```
    fn flatten(
        self,
    ) -> impl ParUseOption<
        Item = <Self::Item as IntoIterator>::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first successful item according to iteration order.
    ///
    /// Returns:
    /// - `None` if computation short-circuits due to a failure
    /// - `Some(None)` if no successful element exists
    /// - `Some(Some(x))` for the first successful element
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..4)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .first();
    /// assert_eq!(ok, Some(Some(1)));
    ///
    /// let fail = vec![None, Some(1), Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .first();
    /// assert_eq!(fail, None);
    /// ```
    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    /// Reduces successful items into one value using `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..6)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .reduce(|_, a, b| a + b);
    /// assert_eq!(ok, Some(Some(15)));
    ///
    /// let fail = vec![Some(1), None, Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .reduce(|_, a, b| a + b);
    /// assert_eq!(fail, None);
    /// ```
    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(&mut Self::Use, Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    /// Collects successful items into `dst`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10usize];
    /// let ok = (0..3)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .collect_into(&mut dst);
    /// assert_eq!(ok, Some(()));
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    /// ```
    fn collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    /// Collects successful items into a new collection.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Option<Vec<_>> = (1..4)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .collect();
    ///
    /// assert_eq!(out, Some(vec![1, 2, 3]));
    /// ```
    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    /// Returns `Some(true)` if all successful items satisfy `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .all(|_, x| x > &0);
    /// assert_eq!(ok, Some(true));
    ///
    /// let fail = vec![Some(1), None, Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .all(|_, x| x > &0);
    /// assert_eq!(fail, None);
    /// ```
    fn all<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| !*x)
            .map(|x| x.is_none())
    }

    /// Returns `Some(true)` if any successful item satisfies `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .any(|_, x| x % 2 == 0);
    /// assert_eq!(ok, Some(true));
    ///
    /// let fail = vec![Some(1), None, Some(3)]
    ///     .into_par()
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .any(|_, x| x % 2 == 0);
    /// assert_eq!(fail, None);
    /// ```
    fn any<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| *x)
            .map(|x| x.is_some())
    }

    /// Counts successful elements.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok = (1..11)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 3 == 0)
    ///     .count();
    /// assert_eq!(ok, Some(3));
    /// ```
    fn count(self) -> Option<usize> {
        self.map(|_, _| 1)
            .reduce(|_, a, b| a + b)
            .map(|x| x.unwrap_or(0))
    }

    /// Finds first (ordered) or any (arbitrary) successful item satisfying `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = (1..101)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .find(|_, x| x % 17 == 0);
    /// assert_eq!(found, Some(Some(17)));
    /// ```
    fn find<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    /// Executes `f` for each successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = UseVec::new(|_| 0usize);
    ///
    /// let result = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_vec(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(result, Some(()));
    /// assert_eq!(sums.into_vec().into_iter().sum::<usize>(), 10);
    /// ```
    fn for_each<F>(self, f: F) -> Option<()>
    where
        F: Fn(&mut Self::Use, Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }

    /// Returns maximum successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let m = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .max();
    /// assert_eq!(m, Some(Some(4)));
    /// ```
    fn max(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns successful element considered maximum by comparator `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .max_by(|_, a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(5)));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with maximum key value.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .max_by_key(|_, x| x.abs());
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns minimum successful element.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let m = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .min();
    /// assert_eq!(m, Some(Some(1)));
    /// ```
    fn min(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns successful element considered minimum by comparator `f`.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .min_by(|_, a, b| a.cmp(b));
    /// assert_eq!(x, Some(Some(-10)));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns successful element with minimum key value.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .min_by_key(|_, x| x.abs());
    /// assert_eq!(x, Some(Some(0)));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums successful elements using [`Sum`] implementation.
    ///
    /// Returns `None` on short-circuit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let s: Option<usize> = (1..5)
    ///     .into_par()
    ///     .map(Some)
    ///     .into_optional()
    ///     .use_new(|_| ())
    ///     .sum();
    /// assert_eq!(s, Some(10));
    /// ```
    fn sum<S>(self) -> Option<S>
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(|_, x| Self::Item::owned(x))
            .reduce(|_, a, b| Self::Item::add(a, b))
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
