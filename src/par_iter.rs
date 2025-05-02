use crate::{
    Params,
    collect_into::ParCollectInto,
    computations::{map_clone, map_copy, map_count, reduce_sum, reduce_unit},
    parameters::{ChunkSize, CollectOrdering, NumThreads},
    runner::{DefaultRunner, ParallelRunner},
    special_type_sets::Sum,
};
use orx_concurrent_iter::ConcurrentIter;
use std::cmp::Ordering;

/// Parallel iterator.
pub trait ParIter<R = DefaultRunner>: Sized + Send + Sync
where
    R: ParallelRunner,
{
    /// Element type of the parallel iterator.
    type Item: Send + Sync;

    /// Returns a reference to the input concurrent iterator.
    fn con_iter(&self) -> &impl ConcurrentIter;

    /// Parameters of the parallel iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).params(),
    ///     &Params::new(1, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).chunk_size(64).params(),
    ///     &Params::new(1, 64, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par()
    ///         .num_threads(1)
    ///         .chunk_size(64)
    ///         .collect_ordering(CollectOrdering::Arbitrary)
    ///         .params(),
    ///     &Params::new(1, 64, CollectOrdering::Arbitrary)
    /// );
    /// ```
    fn params(&self) -> &Params;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    /// See [`NumThreads`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // all available threads can be used
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(0).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// // maximum 4 threads can be used
    /// assert_eq!(
    ///     vec.par().num_threads(4).chunk_size(64).params(),
    ///     &Params::new(4, 64, CollectOrdering::Ordered)
    /// );
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `NumThreads::Min(n)`
    ///
    /// See [`ChunkSize`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::num::NonZero;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // chunk sizes will be dynamically decided by the parallel runner
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().chunk_size(0).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// // minimum chunk size will be 64, but can be dynamically increased by the parallel runner
    /// assert_eq!(
    ///     vec.par().chunk_size(64).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Min(NonZero::new(64).unwrap()), CollectOrdering::Ordered)
    /// );
    ///
    /// // chunk size will always be 64, parallel runner cannot change
    /// assert_eq!(
    ///     vec.par().chunk_size(ChunkSize::Exact(NonZero::new(64).unwrap())).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Exact(NonZero::new(64).unwrap()), CollectOrdering::Ordered)
    /// );
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the collect ordering of the parallel computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// /// results are collected in order consistent to the input order
    /// assert_eq!(
    ///     vec.par().params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().collect_ordering(CollectOrdering::Ordered).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Ordered)
    /// );
    ///
    /// /// results might be collected in arbitrary order
    /// assert_eq!(
    ///     vec.par().collect_ordering(CollectOrdering::Arbitrary).params(),
    ///     &Params::new(NumThreads::Auto, ChunkSize::Auto, CollectOrdering::Arbitrary)
    /// );
    /// ```
    fn collect_ordering(self, collect: CollectOrdering) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// let inputs = vec![1, 2, 3, 4];
    ///
    /// // uses the default runner
    /// let sum = inputs.par().sum();
    ///
    /// // uses the custom parallel runner MyParallelRunner: ParallelRunner
    /// let sum = inputs.par().with_runner::<MyParallelRunner>().sum();
    /// ```
    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item>;

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let iter = a.into_par().map(|x| 2 * x);
    ///
    /// let b: Vec<_> = iter.collect();
    /// assert_eq!(b, &[2, 4, 6]);
    /// ```
    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone;

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let iter = a.into_par().filter(|x| *x % 2 == 1).copied();
    ///
    /// let b: Vec<_> = iter.collect();
    /// assert_eq!(b, &[1, 3]);
    /// ```
    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync + Clone;

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = ["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_chars: Vec<_> = words.into_par().flat_map(|s| s.chars()).collect();
    ///
    /// let merged: String = all_chars.iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator + Send + Sync,
        IOut::IntoIter: Send + Sync,
        IOut::Item: Send + Sync,
        FlatMap: Fn(Self::Item) -> IOut + Send + Sync + Clone;

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    /// The example below shows how a `map().filter().map()` can be shortened to a single call to `filter_map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = ["1", "two", "NaN", "four", "5"];
    ///
    /// let numbers: Vec<_> = a
    ///     .into_par()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, [1, 5]);
    /// ```
    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        FilterMap: Fn(Self::Item) -> Option<Out> + Send + Sync + Clone;

    /// Does something with each element of an iterator, passing the value on.
    ///
    /// When using iterators, you’ll often chain several of them together.
    /// While working on such code, you might want to check out what’s happening at various parts in the pipeline.
    /// To do that, insert a call to `inspect()`.
    ///
    /// It’s more common for `inspect()` to be used as a debugging tool than to exist in your final code,
    /// but applications may find it useful in certain situations when errors need to be logged before being discarded.
    ///
    /// It is often convenient to use thread-safe collections such as [`ConcurrentBag`] and [`ConcurrentVec`] to
    /// collect some intermediate values during parallel execution for further inspection.
    /// The following example demonstrates such a use case.
    ///
    /// [`ConcurrentBag`]: orx_concurrent_bag::ConcurrentBag
    /// [`ConcurrentVec`]: orx_concurrent_vec::ConcurrentVec
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let a = vec![1, 4, 2, 3];
    ///
    /// // let's add some inspect() calls to investigate what's happening
    /// // - log some events
    /// // - use a concurrent bag to collect and investigate numbers contributing to the sum
    /// let bag = ConcurrentBag::new();
    ///
    /// let sum = a
    ///     .par()
    ///     .copied()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// println!("{sum}");
    ///
    /// let mut values_made_through = bag.into_inner();
    /// values_made_through.sort();
    /// assert_eq!(values_made_through, [2, 4]);
    /// ```
    ///
    /// This will print:
    ///
    /// ```console
    /// about to filter: 1
    /// about to filter: 4
    /// made it through filter: 4
    /// about to filter: 2
    /// made it through filter: 2
    /// about to filter: 3
    /// 6
    /// ```
    fn inspect<Operation>(self, operation: Operation) -> impl ParIter<R, Item = Self::Item>
    where
        Operation: Fn(&Self::Item) + Sync + Send + Clone,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // special item transformations

    /// Creates an iterator which copies all of its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let v_copied: Vec<_> = a.par().copied().collect();
    ///
    /// // copied is the same as .map(|&x| x)
    /// let v_map: Vec<_> = a.par().map(|&x| x).collect();
    ///
    /// assert_eq!(v_copied, vec![1, 2, 3]);
    /// assert_eq!(v_map, vec![1, 2, 3]);
    /// ```
    fn copied<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Copy + Send + Sync,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_copy)
    }

    /// Creates an iterator which clones all of its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<_> = [1, 2, 3].map(|x| x.to_string()).into_iter().collect();
    ///
    /// let v_cloned: Vec<_> = a.par().cloned().collect();
    ///
    /// // cloned is the same as .map(|x| x.clone())
    /// let v_map: Vec<_> = a.par().map(|x| x.clone()).collect();
    ///
    /// assert_eq!(
    ///     v_cloned,
    ///     vec![String::from("1"), String::from("2"), String::from("3")]
    /// );
    /// assert_eq!(
    ///     v_map,
    ///     vec![String::from("1"), String::from("2"), String::from("3")]
    /// );
    /// ```
    fn cloned<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Clone + Send + Sync,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_clone)
    }

    /// Creates an iterator that flattens nested structure.
    ///
    /// This is useful when you have an iterator of iterators or an iterator of things that can be
    /// turned into iterators and you want to remove one level of indirection.
    ///
    /// # Examples
    ///
    /// Basic usage.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![vec![1, 2, 3, 4], vec![5, 6]];
    /// let flattened = data.into_par().flatten().collect::<Vec<u8>>();
    /// assert_eq!(flattened, &[1, 2, 3, 4, 5, 6]);
    /// ```
    ///
    /// Mapping and then flattening:
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = vec!["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_characters: Vec<_> = words.par().map(|s| s.chars()).flatten().collect();
    /// let merged: String = all_characters.into_iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    ///
    /// But actually, you can write this in terms of `flat_map`,
    /// which is preferable in this case since it conveys intent more clearly:
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = vec!["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_characters: Vec<_> = words.par().flat_map(|s| s.chars()).collect();
    /// let merged: String = all_characters.into_iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    fn flatten(self) -> impl ParIter<R, Item = <Self::Item as IntoIterator>::Item>
    where
        Self::Item: IntoIterator,
        <Self::Item as IntoIterator>::IntoIter: Send + Sync,
        <Self::Item as IntoIterator>::Item: Send + Sync,
        R: Send + Sync,
        Self: Send + Sync,
    {
        let map = |e: Self::Item| e.into_iter();
        self.flat_map(map)
    }

    // collect

    /// Collects all the items from an iterator into a collection.
    ///
    /// This is useful when you already have a collection and want to add the iterator items to it.
    ///
    /// The collection is passed in as owned value, and returned back with the additional elements.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let vec: Vec<i32> = vec![0, 1];
    /// let vec = a.par().map(|&x| x * 2).collect_into(vec);
    /// let vec = a.par().map(|&x| x * 10).collect_into(vec);
    ///
    /// assert_eq!(vec, vec![0, 1, 2, 4, 6, 10, 20, 30]);
    /// ```
    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    /// Transforms an iterator into a collection.
    ///
    /// Similar to [`Iterator::collect`], the type annotation on the left-hand-side determines
    /// the type of the result collection; or turbofish annotation can be used.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let doubled: Vec<i32> = a.par().map(|&x| x * 2).collect();
    ///
    /// assert_eq!(vec![2, 4, 6], doubled);
    /// ```
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    ///
    /// If the iterator is empty, returns `None`; otherwise, returns the result of the reduction.
    ///
    /// The `reduce` function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let inputs = 1..10;
    /// let reduced: usize = inputs.par().reduce(|acc, e| acc + e).unwrap_or(0);
    /// assert_eq!(reduced, 45);
    /// ```
    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync;

    /// Tests if every element of the iterator matches a predicate.
    ///
    /// `all` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if they all return true, then so does `all`.
    /// If any of them returns false, it returns false.
    ///
    /// `all` is short-circuiting; in other words, it will stop processing as soon as it finds a false,
    /// given that no matter what else happens, the result will also be false.
    ///
    /// An empty iterator returns true.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut a = vec![1, 2, 3];
    /// assert!(a.par().all(|x| **x > 0));
    /// assert!(!a.par().all(|x| **x > 2));
    ///
    /// a.clear();
    /// assert!(a.par().all(|x| **x > 2)); // empty iterator
    /// ```
    fn all<Predicate>(self, predicate: Predicate) -> bool
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).is_none()
    }

    /// Tests if any element of the iterator matches a predicate.
    ///
    /// `any` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if any of the elements returns true, then so does `any`.
    /// If all of them return false, it returns false.
    ///
    /// `any` is short-circuiting; in other words, it will stop processing as soon as it finds a true,
    /// given that no matter what else happens, the result will also be true.
    ///
    /// An empty iterator returns false.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut a = vec![1, 2, 3];
    /// assert!(a.par().any(|x| **x > 0));
    /// assert!(!a.par().any(|x| **x > 5));
    ///
    /// a.clear();
    /// assert!(!a.par().any(|x| **x > 0)); // empty iterator
    /// ```
    fn any<Predicate>(self, predicate: Predicate) -> bool
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.find(predicate).is_some()
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// assert_eq!(a.par().filter(|x| **x >= 2).count(), 2);
    /// ```
    fn count(self) -> usize {
        self.map(map_count).reduce(reduce_sum).unwrap_or(0)
    }

    /// Calls a closure on each element of an iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::sync::mpsc::channel;
    ///
    /// let (tx, rx) = channel();
    /// (0..5)
    ///     .par()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// let mut v: Vec<_> = rx.iter().collect();
    /// v.sort(); // order can be mixed, since messages will be sent in parallel
    /// assert_eq!(v, vec![1, 3, 5, 7, 9]);
    /// ```
    ///
    /// Note that since parallel iterators cannot be used within the `for` loop as regular iterators,
    /// `for_each` provides a way to perform arbitrary for loops on parallel iterators.
    /// In the following example, we log every element that satisfies a predicate in parallel.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// (0..5)
    ///     .par()
    ///     .flat_map(|x| x * 100..x * 110)
    ///     .filter(|&x| x % 3 == 0)
    ///     .for_each(|x| println!("{x}"));
    /// ```
    fn for_each<Operation>(self, operation: Operation)
    where
        Operation: Fn(Self::Item) + Sync + Send,
    {
        let map = |x| operation(x);
        let _ = self.map(map).reduce(reduce_unit);
    }

    /// Returns the maximum element of an iterator.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.par().max(), Some(&3));
    /// assert_eq!(b.par().max(), None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.reduce(Ord::max)
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().max_by(|x, y| x.cmp(y)).unwrap(), 5);
    /// ```
    fn max_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().max_by_key(|x| x.abs()).unwrap(), -10);
    /// ```
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Self::Item>
    where
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the minimum element of an iterator.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.par().min(), Some(&1));
    /// assert_eq!(b.par().min(), None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.reduce(Ord::min)
    }

    /// Returns the element that gives the minimum value with respect to the specified `compare` function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().min_by(|x, y| x.cmp(y)).unwrap(), -10);
    /// ```
    fn min_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().min_by_key(|x| x.abs()).unwrap(), 0);
    /// ```
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Self::Item>
    where
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums the elements of an iterator.
    ///
    /// Takes each element, adds them together, and returns the result.
    ///
    /// An empty iterator returns the additive identity (“zero”) of the type, which is 0 for integers and -0.0 for floats.
    ///
    /// `sum` can be used to sum any type implementing [`Sum<Out>`].
    ///
    /// [`Sum<Out>`]: crate::Sum
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let sum: i32 = a.par().sum();
    ///
    /// assert_eq!(sum, 6);
    /// ```
    fn sum<Out>(self) -> Out
    where
        Self::Item: Sum<Out>,
        Out: Send + Sync,
    {
        self.map(Self::Item::map)
            .reduce(Self::Item::reduce)
            .unwrap_or(Self::Item::zero())
    }

    // early exit

    /// Returns the first element of the iterator; returns None if it is empty.
    ///
    /// See also [`any_element`] to fetch any of the elements rather than strictly the first.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<usize> = vec![];
    /// assert_eq!(a.par().copied().first(), None);
    ///
    /// let a = vec![1, 2, 3];
    /// assert_eq!(a.par().copied().first(), Some(1));
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().filter(|x| x % 3421 == 0).first(), Some(3421));
    /// assert_eq!(a.par().filter(|x| x % 12345 == 0).first(), None);
    ///
    /// // or equivalently,
    /// assert_eq!(a.par().find(|x| x % 3421 == 0), Some(3421));
    /// ```
    fn first(self) -> Option<Self::Item>;

    /// Returns any element of the iterator; returns None if it is empty.
    ///
    /// This is the counterpart of [`first`] where it is okay to fetch any of the elements
    /// of the iterator, rather than the first.
    ///
    /// This is useful specifically when we are searching for any element that satisfies a
    /// desired condition, such as:
    ///
    /// * a feasible solution among all possible solutions,
    /// * an item which is cheaper than a given price,
    /// * a movie with at least 4.8 rating,
    /// * etc.
    ///
    /// [`first`]: crate::ParIter::first
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<usize> = vec![];
    /// assert_eq!(a.par().copied().any_element(), None);
    ///
    /// // might return any of 1, 2 or 3
    /// let a = vec![1, 2, 3];
    /// let any = a.par().copied().any_element().unwrap();
    /// assert!(a.contains(&any));
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().filter(|x| x % 12345 == 0).any_element(), None);
    ///
    /// // might return either of 3421 or 2*3421
    /// let any = a.par().filter(|x| x % 3421 == 0).any_element().unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    ///
    /// // or equivalently,
    /// let any = a.par().find_any(|x| x % 3421 == 0).unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    /// ```
    fn any_element(self) -> Option<Self::Item>;

    /// Searches for an element of an iterator that satisfies a `predicate`.
    ///
    /// `find` takes a closure that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and returns `Some(x)` where `x` is the first element that returns true.
    /// If they all return false, it returns None.
    ///
    /// `find` is short-circuiting; in other words, it will stop processing as soon as the closure returns true.
    ///
    /// `par_iter.find(predicate)` can also be considered as a shorthand for `par_iter.filter(predicate).first()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().find(|x| x % 12345 == 0), None);
    /// assert_eq!(a.par().find(|x| x % 3421 == 0), Some(3421));
    /// ```
    fn find<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).first()
    }

    /// Searches for an element of an iterator that satisfies a `predicate`.
    ///
    /// `find_any` takes a closure that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and returns `Some(x)` where `x` is any of the elements that returns true.
    /// If they all return false, it returns None.
    ///
    /// `find_any` is short-circuiting; in other words, it will stop processing as soon as the closure returns true.
    ///
    /// `par_iter.find_any(predicate)` can also be considered as a shorthand for `par_iter.filter(predicate).any_element()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().find_any(|x| x % 12345 == 0), None);
    ///
    /// // might return either of 3421 or 2*3421
    /// let any = a.par().find_any(|x| x % 3421 == 0).unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    /// ```
    fn find_any<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Predicate: Fn(&Self::Item) -> bool + Send + Sync + Clone,
    {
        self.filter(predicate).any_element()
    }
}
