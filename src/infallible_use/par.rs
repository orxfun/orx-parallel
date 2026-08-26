#![allow(clippy::type_complexity)]

use crate::common_par_traits::ParInfCommon;
use crate::infallible::xap_variants::Id;
use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::xap::FlattenOf;
use crate::infallible_use::xap_variants::{IdUse, UDummyPair};
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, ParUseCore, ParUseIter, XapUse,
};
use crate::option_use::ParUseOptionIter;
use crate::result_use::ParUseResultIter;
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::use_var::{PairPtr, UseFold};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseOption, ParUseResult, Sum,
};
use alloc::vec::Vec;
use core::cmp::Ordering;
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator pipelines with worker-local mutable state.
///
/// `ParUse` extends the usual parallel iterator pipeline with an associated
/// [`Use`](crate::Use) value that is passed into transformation and reduction
/// steps as mutable worker-local state. This is useful when each worker needs
/// its own reusable scratch space, accumulator, or other per-thread context.
///
/// You can enter this mode from [`Par`](crate::Par) via
/// [`use_new`](crate::Par::use_new),
/// [`use_vec`](crate::Par::use_vec), or
/// [`use_slice`](crate::Par::use_slice).
///
/// Related traits:
/// - [`Par`](crate::Par) for pipelines without worker-local state,
/// - [`ParUseOption`](crate::ParUseOption) via [`into_optional`](crate::ParUse::into_optional),
/// - [`ParUseResult`](crate::ParUseResult) via [`into_fallible`](crate::ParUse::into_fallible).
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let n = 10_000usize;
/// let mut partial_sums = UseVec::new(|_| 0usize);
///
/// (0..n)
///     .into_par()
///     .use_vec(&mut partial_sums)
///     .for_each(|thread_sum, x| *thread_sum += x);
///
/// let total: usize = partial_sums.into_vec().into_iter().sum();
/// assert_eq!(total, (n - 1) * n / 2);
/// ```
///
/// Using an RNG as mutable worker-local state:
///
/// ```
/// use orx_parallel::*;
/// use rand::prelude::*;
/// use rand_chacha::ChaCha8Rng;
///
/// let values: Vec<_> = (0..128usize)
///     .into_par()
///     .use_new(|thread_idx| ChaCha8Rng::seed_from_u64(42 + thread_idx as u64))
///     .map(|rng, x| x + rng.random_range(0..10))
///     .collect();
///
/// assert_eq!(values.len(), 128);
/// assert!(
///     values
///         .iter()
///         .enumerate()
///         .all(|(i, v)| *v >= i && *v < i + 10)
/// );
/// ```
pub trait ParUse: Sized + ParUseCore + ParInfCommon<CommonItem = Self::Item> {
    // configuration

    /// Replaces the current parallel runner with `runner`.
    ///
    /// This allows per-computation control over execution strategy.
    ///
    /// Please see [`Runner`](crate::Runner) for available runners.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = (1..101).par().use_new(|_| ());
    ///
    /// let par = par.runner(Runner::fixed());
    ///
    /// let sum = par.sum();
    /// assert_eq!(sum, 5050);
    /// ```
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input>;

    #[cfg(feature = "std")]
    /// Wraps the current runner with diagnostics-enabled execution.
    ///
    /// The resulting pipeline behaves the same, while also printing runtime
    /// diagnostics at the end.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let par = (1..1001).par().use_new(|_| ());
    ///
    /// #[cfg(feature = "std")]
    /// let par = par.runner_with_diagnostics();
    ///
    /// let sum = par.sum();
    /// assert_eq!(sum, 500500);
    /// ```
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input>;

    /// Sets the maximum number of worker threads for this computation.
    ///
    /// Integer values map as follows:
    /// - `0` => automatic (default)
    /// - `n > 0` => at most `n` threads
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: usize = (1..11)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .num_threads(1)
    ///     .sum();
    ///
    /// assert_eq!(sum, 55);
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets chunk size used when pulling items from the concurrent input.
    ///
    /// Integer values map as follows:
    /// - `0` => automatic (default)
    /// - `n > 0` => exact chunk size `n`
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let values: Vec<_> = (0..16)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .chunk_size(4)
    ///     .map(|_, x| x + 1)
    ///     .collect();
    ///
    /// assert_eq!(values.len(), 16);
    /// assert_eq!(values[0], 1);
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets iteration order semantics for order-sensitive operations.
    ///
    /// With `Ordered` (default), methods like `first` and `find` follow input
    /// position. With `Arbitrary`, any matching item found first in parallel
    /// execution may be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ordered = (1..10_000)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .iteration_order(IterationOrder::Ordered)
    ///     .find(|_, x| x % 3421 == 0);
    /// assert_eq!(ordered, Some(3421));
    ///
    /// let any = (1..10_000)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .iteration_order(IterationOrder::Arbitrary)
    ///     .find(|_, x| x % 3421 == 0)
    ///     .unwrap();
    /// assert!([3421, 6842].contains(&any));
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    /// Converts `ParUse<Item = Option<T>>` into [`ParUseOption`](crate::ParUseOption).
    ///
    /// The result short-circuits to `None` if any element is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Option<Vec<_>> = ["1", "2", "3"]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, s| s.parse::<i32>().ok())
    ///     .into_optional()
    ///     .map(|_, x| x * 2)
    ///     .collect();
    /// assert_eq!(ok, Some(vec![2, 4, 6]));
    ///
    /// let fail: Option<Vec<_>> = ["1", "x", "3"]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, s| s.parse::<i32>().ok())
    ///     .into_optional()
    ///     .map(|_, x| x * 2)
    ///     .collect();
    /// assert_eq!(fail, None);
    /// ```
    fn into_optional<T>(
        self,
    ) -> impl ParUseOption<
        Item = T,
        Use = Self::Use,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = IdUse<Id<T>, Self::Use>,
        Input = Self::Input,
        Size = <<Self::Xap as XapUse>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: XapUse<U = Self::Use, I = <Self::Input as ConcurrentIter>::Item, O = Option<T>>,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseOptionIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    /// Converts `ParUse<Item = Result<T, E>>` into [`ParUseResult`](crate::ParUseResult).
    ///
    /// The result short-circuits and returns the first observed error.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let ok: Result<Vec<_>, _> = ["1", "2", "3"]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, s| s.parse::<i32>())
    ///     .into_fallible()
    ///     .map(|_, x| x * 2)
    ///     .collect();
    /// assert_eq!(ok, Ok(vec![2, 4, 6]));
    ///
    /// let fail: Result<Vec<_>, _> = ["1", "x", "3"]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, s| s.parse::<i32>())
    ///     .into_fallible()
    ///     .map(|_, x| x * 2)
    ///     .collect();
    /// assert!(fail.is_err());
    /// ```
    fn into_fallible<T, E>(
        self,
    ) -> impl ParUseResult<
        Item = T,
        Error = E,
        Use = Self::Use,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = IdUse<Id<T>, Self::Use>,
        Input = Self::Input,
        Size = <<Self::Xap as XapUse>::Size as Size>::IntoPair,
    >
    where
        Self::Xap:
            XapUse<U = Self::Use, I = <Self::Input as ConcurrentIter>::Item, O = Result<T, E>>,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseResultIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    /// Copies elements of a reference iterator.
    ///
    /// Equivalent to `.map(|_, &x| x)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![1, 2, 3];
    /// let copied: Vec<_> = data.par().use_new(|_| ()).copied().collect();
    ///
    /// assert_eq!(copied, vec![1, 2, 3]);
    /// ```
    fn copied<'a, O>(
        self,
    ) -> impl ParUse<
        Item = O,
        Use = Self::Use,
        Xap = MappedOf<Self::Xap, UFnCopied<'a, Self::Use, O>>,
        Input = Self::Input,
    >
    where
        Self: ParUse<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseIter::new(u, iter, xap.mapped(UFnCopied::new()), exe, params)
    }

    /// Clones elements of a reference iterator.
    ///
    /// Equivalent to `.map(|_, x| x.clone())`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec!["a".to_string(), "b".to_string()];
    /// let cloned: Vec<_> = data.par().use_new(|_| ()).cloned().collect();
    ///
    /// assert_eq!(cloned, vec!["a".to_string(), "b".to_string()]);
    /// ```
    fn cloned<'a, O>(
        self,
    ) -> impl ParUse<
        Item = O,
        Use = Self::Use,
        Xap = MappedOf<Self::Xap, UFnCloned<'a, Self::Use, O>>,
        Input = Self::Input,
    >
    where
        Self: ParUse<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseIter::new(u, iter, xap.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    /// Maps each element with closure `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let doubled: Vec<_> = (1..4)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, x| 2 * x)
    ///     .collect();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send;

    /// Runs `h` on each element and forwards the item unchanged.
    ///
    /// Useful for logging, metrics, and tracing with worker-local state.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut calls = UseVec::new(|_| 0usize);
    ///
    /// let out: Vec<_> = (1..5)
    ///     .into_par()
    ///     .use_vec(&mut calls)
    ///     .inspect(|count, _| *count += 1)
    ///     .collect();
    ///
    /// assert_eq!(out, vec![1, 2, 3, 4]);
    /// assert_eq!(calls.into_vec().into_iter().sum::<usize>(), 4);
    /// ```
    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send;

    /// Keeps only elements satisfying predicate `h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let odds: Vec<_> = (1..7)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 2 == 1)
    ///     .collect();
    ///
    /// assert_eq!(odds, vec![1, 3, 5]);
    /// ```
    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send;

    /// Maps and filters in a single pass.
    ///
    /// Returns mapped values for elements where `h` returns `Some(_)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nums: Vec<_> = ["1", "x", "5"]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .filter_map(|_, s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(nums, vec![1, 5]);
    /// ```
    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send;

    /// Maps each element to an iterator and flattens one level.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = (1..4)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .flat_map(|_, x| [x, x + 10])
    ///     .collect();
    ///
    /// assert_eq!(out, vec![1, 11, 2, 12, 3, 13]);
    /// ```
    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUse<
        Item = V::Item,
        Use = Self::Use,
        Xap = FlatMapOf<Self::Xap, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send;

    /// Flattens one level of nested iterables.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let nested = vec![vec![1, 2], vec![3, 4]];
    /// let flat: Vec<_> = nested.into_par().use_new(|_| ()).flatten().collect();
    ///
    /// assert_eq!(flat, vec![1, 2, 3, 4]);
    /// ```
    fn flatten(
        self,
    ) -> impl ParUse<
        Item = <Self::Item as IntoIterator>::Item,
        Use = Self::Use,
        Xap = FlattenOf<Self::Xap>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    /// Returns the first item according to iteration order, or `None` if empty.
    ///
    /// With [`IterationOrder::Ordered`] (default), this is the earliest matching
    /// item by input position. With [`IterationOrder::Arbitrary`], this may be
    /// any matching item reached first in parallel execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!(Vec::<usize>::new().into_par().use_new(|_| ()).first(), None);
    /// assert_eq!((1..4).into_par().use_new(|_| ()).first(), Some(1));
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    /// Reduces items into one value using reducer `f`.
    ///
    /// Returns `None` for an empty iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let reduced = (1..6)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .reduce(|_, a, b| a + b);
    ///
    /// assert_eq!(reduced, Some(15));
    /// ```
    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(&mut Self::Use, Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    /// Collects all items into `dst`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut dst = vec![10];
    /// (0..3).into_par().use_new(|_| ()).collect_into(&mut dst);
    /// assert_eq!(dst, vec![10, 0, 1, 2]);
    /// ```
    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    /// Collects all items into a new collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let out: Vec<_> = (1..4)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .map(|_, x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(out, vec![2, 4, 6]);
    /// ```
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    /// Returns `true` if all items satisfy predicate `f`.
    ///
    /// Empty iterators return `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert!((1..5).into_par().use_new(|_| ()).all(|_, x| x > &0));
    /// assert!(!(1..5).into_par().use_new(|_| ()).all(|_, x| x % 2 == 0));
    /// ```
    fn all<F>(self, f: F) -> bool
    where
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x)).find(|_, x| !*x).is_none()
    }

    /// Returns `true` if any item satisfies predicate `f`.
    ///
    /// Empty iterators return `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert!((1..5).into_par().use_new(|_| ()).any(|_, x| x % 2 == 0));
    /// assert!(!(1..5).into_par().use_new(|_| ()).any(|_, x| x > &10));
    /// ```
    fn any<F>(self, f: F) -> bool
    where
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x)).find(|_, x| *x).is_some()
    }

    /// Counts elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let n = (1..11)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .filter(|_, x| x % 3 == 0)
    ///     .count();
    ///
    /// assert_eq!(n, 3);
    /// ```
    fn count(self) -> usize {
        self.map(|_, _| 1).reduce(|_, a, b| a + b).unwrap_or(0)
    }

    /// Finds first (ordered) or any (arbitrary) item satisfying predicate `f`.
    ///
    /// Equivalent to `self.filter(f).first()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let found = (1..101)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .find(|_, x| x % 17 == 0);
    ///
    /// assert_eq!(found, Some(17));
    /// ```
    fn find<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    /// Folds elements into per-thread accumulators and returns them.
    ///
    /// The output contains one accumulator for each participating worker.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let num_threads = 4;
    /// let partials: Vec<usize> = (1..6)
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .num_threads(num_threads)
    ///     .fold(|| 0usize, |_, acc, x| *acc += x);
    ///
    /// assert!(partials.len() <= num_threads);
    /// assert_eq!(partials.iter().sum::<usize>(), 15);
    /// ```
    fn fold<B, I, F>(self, init: I, f: F) -> Vec<B>
    where
        B: Send,
        I: Fn() -> B + Sync,
        F: Fn(&mut Self::Use, &mut B, Self::Item) + Copy + Send,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        let mut use_fold = UseFold::new(u, |_| init());
        let xap = UDummyPair::<Self::Xap, B>::new(xap);
        let par = ParUseIter::new(&mut use_fold, iter, xap, exe, params);

        par.for_each(move |a: &mut PairPtr<Self::Use, B>, x| {
            let (u, v) = a.u_v_mut();
            f(u, v, x);
        });

        use_fold.into_vec()
    }

    /// Executes `f` for each item.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut sums = UseVec::new(|_| 0usize);
    ///
    /// (1..5)
    ///     .into_par()
    ///     .use_vec(&mut sums)
    ///     .for_each(|local, x| *local += x);
    ///
    /// assert_eq!(sums.into_vec().into_iter().sum::<usize>(), 10);
    /// ```
    fn for_each<F>(self, f: F)
    where
        F: Fn(&mut Self::Use, Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _, _| {});
    }

    /// Returns maximum element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().use_new(|_| ()).max(), Some(4));
    /// assert_eq!(Vec::<usize>::new().into_par().use_new(|_| ()).max(), None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    /// Returns element considered maximum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::cmp::Ordering;
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .max_by(|_, a, b| a.cmp(b));
    /// assert_eq!(x, Some(5));
    /// ```
    fn max_by<F>(self, f: F) -> Option<Self::Item>
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

    /// Returns element with maximum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .max_by_key(|_, x| x.abs());
    /// assert_eq!(x, Some(-10));
    /// ```
    fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    /// Returns minimum element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// assert_eq!((1..5).into_par().use_new(|_| ()).min(), Some(1));
    /// assert_eq!(Vec::<usize>::new().into_par().use_new(|_| ()).min(), None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    /// Returns element considered minimum by comparator `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .min_by(|_, a, b| a.cmp(b));
    /// assert_eq!(x, Some(-10));
    /// ```
    fn min_by<F>(self, f: F) -> Option<Self::Item>
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

    /// Returns element with minimum key value.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let x = vec![-3_i32, 0, 1, 5, -10]
    ///     .into_par()
    ///     .use_new(|_| ())
    ///     .min_by_key(|_, x| x.abs());
    /// assert_eq!(x, Some(0));
    /// ```
    fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    /// Sums elements using [`Sum`] implementation of the item type.
    ///
    /// Empty iterators return additive identity (`zero`).
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let sum: usize = (1..5).into_par().use_new(|_| ()).sum();
    /// assert_eq!(sum, 10);
    /// ```
    fn sum<S>(self) -> S
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(|_, x| Self::Item::owned(x))
            .reduce(|_, a, b| Self::Item::add(a, b))
            .unwrap_or(Self::Item::zero())
    }
}
