use crate::{DefaultRunner, Params, computational_variants::Par};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, ConcurrentRecursiveIterExact};

// unknown size

/// Trait to convert an iterator into a recursive parallel iterator together with the `extend` method.
///
/// Created parallel iterator is a regular parallel iterator; i.e., we have access to
/// all [`ParIter`] features.
///
/// It is recursive due to the extension. The recursive parallel iterator will yield
/// * all initial elements contained in this iterator,
/// * all elements created by calling `extend` on each of the initial elements, let's call these depth-1 elements,
/// * all elements created by calling `extend` on each of the depth-1 elements, let's call these depth-2 elements,
/// * ..., and so on.
///
/// You may read more about the [`ConcurrentRecursiveIterCore`].
///
/// See also [`IntoParIterRecExact`]
pub trait IntoParIterRec
where
    Self: IntoIterator,
    Self::Item: Send,
{
    /// Converts this iterator into a recursive parallel iterator together with the `extend`
    /// method.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to
    /// all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements created by calling `extend` on each of the initial elements, let's call these depth-1 elements,
    /// * all elements created by calling `extend` on each of the depth-1 elements, let's call these depth-2 elements,
    /// * ..., and so on.
    ///
    /// You may read more about the [`ConcurrentRecursiveIterCore`].
    ///
    /// See also [`IntoParIterRecExact`]
    ///
    /// # Examples
    ///
    /// The following example has some code to set up until the `# usage` line. Notice that the `Node`
    /// is a recursive data structure with children being other nodes.
    ///
    /// We have three initial elements `roots`.
    ///
    /// We want to compute is the sum of Fibonacci numbers of values of all nodes descending from the
    /// roots.
    ///
    /// The `expand` function defines the recursive expansion behavior:
    /// * every process node first adds its children to the end of the iterator,
    /// * then, once they are process, we will create the children of these children as well,
    /// * this process will recursively continue until there is no unprocessed node left.
    ///
    /// This crate makes use of the [`ConcurrentRecursiveIter`] and [`ConcurrentRecursiveIterExact`]
    /// for this computation and provides three ways to execute this computation in parallel.
    ///
    /// ## A. Recursive Iterator with Exact Length
    ///
    /// If we know, or if it is possible and sufficiently cheap to find out, the exact length of the iterator,
    /// it is recommended to work with exact length recursive iterator. Note that the exact length of an
    /// iterator is the total of all elements that will be created. This gives the parallel executor
    /// opportunity to optimize the chunk sizes.
    ///
    /// In the following example, we first calculate this number with `roots.iter().map(|x| x.seq_num_nodes()).sum();`
    /// and then use `(&roots).into_par_rec_exact(extend, count)` to create our exact length recursive parallel
    /// iterator.
    ///
    /// Note that, once we create the recursive parallel iterator, it is just another [`ParIter`]. In other words,
    /// we have access to all parallel iterator features.
    ///
    /// ## B. Recursive Iterator with Unknown Length
    ///
    /// If we cannot know or it is expensive to know the exact length of the iterator ahead of time, we can
    /// still create a recursive parallel iterator. In these cases; however, it is recommended to provide
    /// chunk size explicitly depending on the number of threads that will be used and any estimate on the exact
    /// length.
    ///
    /// In the following example, we directly create the parallel iterator with `(&roots).into_par_rec(extend)`
    /// without providing any length information. Then, we ask the parallel executor to pull tasks in chunks of
    /// 1024 with `.chunk_size(1024)`. Recall the general rule-of-thumb on chunk size parameter:
    /// * the longer each individual computation, the smaller the chunks can be,
    /// * when it is too small, we might suffer from parallelization overhead,
    /// * when it is too large, we might suffer from heterogeneity of tasks which might lead to imbalance of
    ///   load of threads,
    /// * we might try to set it to a large enough value to reduce parallelization overhead without causing
    ///   imbalance.
    ///
    /// ## C. Into Eager Transformation
    ///
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to flatten the tasks and then perform the parallel computation.
    ///
    /// This might increase performance in certain cases; however, requires storing the flattened tasks.
    /// Therefore, it fits best to situations where the input elements are not very large.
    /// In the following example, for instance, elements are of type `&Node` which is a pointer size
    /// which makes it suitable for this approach.
    ///
    /// Note that exact size will be obtained during flattening; and hence, we do not need to provide the
    /// `count`.
    ///
    /// In the example, we create eagerly flattened parallel iterator with the
    /// `(&roots).into_par_rec(extend).into_eager()` call.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIterCore`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIterCore
    ///
    /// ## Example with all three approaches
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha8Rng;
    ///
    /// struct Node {
    ///     value: Vec<u64>,
    ///     children: Vec<Node>,
    /// }
    ///
    /// impl Node {
    ///     fn new(mut n: u32, rng: &mut impl Rng) -> Self {
    ///         let mut children = Vec::new();
    ///         if n < 5 {
    ///             for _ in 0..n {
    ///                 children.push(Node::new(0, rng));
    ///             }
    ///         } else {
    ///             while n > 0 {
    ///                 let n2 = rng.random_range(0..=n);
    ///                 children.push(Node::new(n2, rng));
    ///                 n -= n2;
    ///             }
    ///         }
    ///         Self {
    ///             value: (0..rng.random_range(1..500))
    ///                 .map(|_| rng.random_range(0..40))
    ///                 .collect(),
    ///             children,
    ///         }
    ///     }
    ///
    ///     fn seq_num_nodes(&self) -> usize {
    ///         1 + self
    ///             .children
    ///             .iter()
    ///             .map(|node| node.seq_num_nodes())
    ///             .sum::<usize>()
    ///     }
    ///
    ///     fn seq_sum_fib(&self) -> u64 {
    ///         self.value.iter().map(|x| fibonacci(*x)).sum::<u64>()
    ///             + self.children.iter().map(|x| x.seq_sum_fib()).sum::<u64>()
    ///     }
    /// }
    ///
    /// fn fibonacci(n: u64) -> u64 {
    ///     let mut a = 0;
    ///     let mut b = 1;
    ///     for _ in 0..n {
    ///         let c = a + b;
    ///         a = b;
    ///         b = c;
    ///     }
    ///     a
    /// }
    ///
    /// // # usage
    ///
    /// // this defines how the iterator must extend:
    /// // each node drawn from the iterator adds its children to the end of the iterator
    /// fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
    ///     &node.children
    /// }
    ///
    /// let mut rng = ChaCha8Rng::seed_from_u64(42);
    /// let roots = vec![
    ///     Node::new(50, &mut rng),
    ///     Node::new(20, &mut rng),
    ///     Node::new(40, &mut rng),
    /// ];
    ///
    /// let seq_sum: u64 = roots.iter().map(|x| x.seq_sum_fib()).sum();
    ///
    /// // A. exact length, recommended when possible
    ///
    /// let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();
    ///
    /// let sum = (&roots)
    ///     .into_par_rec_exact(extend, count)
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    ///
    /// // B. guide the computation with chunk size, when length is unknown
    ///
    /// let sum = (&roots)
    ///     .into_par_rec(extend)
    ///     .chunk_size(1024)
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    ///
    /// // C. eagerly convert to a flat iterator
    ///
    /// let sum = (&roots)
    ///     .into_par_rec(extend)
    ///     .into_eager()
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    /// ```
    fn into_par_rec<E, I>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&Self::Item) -> I + Sync;
}

impl<X> IntoParIterRec for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_rec<E, I>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&Self::Item) -> I + Sync,
    {
        let con_rec_iter = ConcurrentRecursiveIter::new(extend, self);
        Par::new(DefaultRunner::default(), Params::default(), con_rec_iter)
    }
}

// exact size

/// Trait to convert an iterator into an exact-sized recursive parallel iterator together with the `extend` method
/// and `exact_len`,
///
/// Created parallel iterator is a regular parallel iterator; i.e., we have access to
/// all [`ParIter`] features.
///
/// It is recursive due to the extension. The recursive parallel iterator will yield
/// * all initial elements contained in this iterator,
/// * all elements created by calling `extend` on each of the initial elements, let's call these depth-1 elements,
/// * all elements created by calling `extend` on each of the depth-1 elements, let's call these depth-2 elements,
/// * ..., and so on.
///
/// You may read more about the [`ConcurrentRecursiveIterCore`].
///
/// See also [`IntoParIterRec`]
pub trait IntoParIterRecExact
where
    Self: IntoIterator,
    Self::Item: Send,
{
    /// Converts this iterator into a recursive parallel iterator together with the `extend` method and `exact_len`.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements created by calling `extend` on each of the initial elements, let's call these depth-1 elements,
    /// * all elements created by calling `extend` on each of the depth-1 elements, let's call these depth-2 elements,
    /// * ..., and so on.
    ///
    /// You may read more about the [`ConcurrentRecursiveIterCore`].
    ///
    /// See also [`IntoParIterRec`]
    ///
    /// # Examples
    ///
    /// The following example has some code to set up until the `# usage` line. Notice that the `Node`
    /// is a recursive data structure with children being other nodes.
    ///
    /// We have three initial elements `roots`.
    ///
    /// We want to compute is the sum of Fibonacci numbers of values of all nodes descending from the
    /// roots.
    ///
    /// The `expand` function defines the recursive expansion behavior:
    /// * every process node first adds its children to the end of the iterator,
    /// * then, once they are process, we will create the children of these children as well,
    /// * this process will recursively continue until there is no unprocessed node left.
    ///
    /// This crate makes use of the [`ConcurrentRecursiveIter`] and [`ConcurrentRecursiveIterExact`]
    /// for this computation and provides three ways to execute this computation in parallel.
    ///
    /// ## A. Recursive Iterator with Exact Length
    ///
    /// If we know, or if it is possible and sufficiently cheap to find out, the exact length of the iterator,
    /// it is recommended to work with exact length recursive iterator. Note that the exact length of an
    /// iterator is the total of all elements that will be created. This gives the parallel executor
    /// opportunity to optimize the chunk sizes.
    ///
    /// In the following example, we first calculate this number with `roots.iter().map(|x| x.seq_num_nodes()).sum();`
    /// and then use `(&roots).into_par_rec_exact(extend, count)` to create our exact length recursive parallel
    /// iterator.
    ///
    /// Note that, once we create the recursive parallel iterator, it is just another [`ParIter`]. In other words,
    /// we have access to all parallel iterator features.
    ///
    /// ## B. Recursive Iterator with Unknown Length
    ///
    /// If we cannot know or it is expensive to know the exact length of the iterator ahead of time, we can
    /// still create a recursive parallel iterator. In these cases; however, it is recommended to provide
    /// chunk size explicitly depending on the number of threads that will be used and any estimate on the exact
    /// length.
    ///
    /// In the following example, we directly create the parallel iterator with `(&roots).into_par_rec(extend)`
    /// without providing any length information. Then, we ask the parallel executor to pull tasks in chunks of
    /// 1024 with `.chunk_size(1024)`. Recall the general rule-of-thumb on chunk size parameter:
    /// * the longer each individual computation, the smaller the chunks can be,
    /// * when it is too small, we might suffer from parallelization overhead,
    /// * when it is too large, we might suffer from heterogeneity of tasks which might lead to imbalance of
    ///   load of threads,
    /// * we might try to set it to a large enough value to reduce parallelization overhead without causing
    ///   imbalance.
    ///
    /// ## C. Into Eager Transformation
    ///
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to flatten the tasks and then perform the parallel computation.
    ///
    /// This might increase performance in certain cases; however, requires storing the flattened tasks.
    /// Therefore, it fits best to situations where the input elements are not very large.
    /// In the following example, for instance, elements are of type `&Node` which is a pointer size
    /// which makes it suitable for this approach.
    ///
    /// Note that exact size will be obtained during flattening; and hence, we do not need to provide the
    /// `count`.
    ///
    /// In the example, we create eagerly flattened parallel iterator with the
    /// `(&roots).into_par_rec(extend).into_eager()` call.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIterCore`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIterCore
    ///
    /// ## Example with all three approaches
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha8Rng;
    ///
    /// struct Node {
    ///     value: Vec<u64>,
    ///     children: Vec<Node>,
    /// }
    ///
    /// impl Node {
    ///     fn new(mut n: u32, rng: &mut impl Rng) -> Self {
    ///         let mut children = Vec::new();
    ///         if n < 5 {
    ///             for _ in 0..n {
    ///                 children.push(Node::new(0, rng));
    ///             }
    ///         } else {
    ///             while n > 0 {
    ///                 let n2 = rng.random_range(0..=n);
    ///                 children.push(Node::new(n2, rng));
    ///                 n -= n2;
    ///             }
    ///         }
    ///         Self {
    ///             value: (0..rng.random_range(1..500))
    ///                 .map(|_| rng.random_range(0..40))
    ///                 .collect(),
    ///             children,
    ///         }
    ///     }
    ///
    ///     fn seq_num_nodes(&self) -> usize {
    ///         1 + self
    ///             .children
    ///             .iter()
    ///             .map(|node| node.seq_num_nodes())
    ///             .sum::<usize>()
    ///     }
    ///
    ///     fn seq_sum_fib(&self) -> u64 {
    ///         self.value.iter().map(|x| fibonacci(*x)).sum::<u64>()
    ///             + self.children.iter().map(|x| x.seq_sum_fib()).sum::<u64>()
    ///     }
    /// }
    ///
    /// fn fibonacci(n: u64) -> u64 {
    ///     let mut a = 0;
    ///     let mut b = 1;
    ///     for _ in 0..n {
    ///         let c = a + b;
    ///         a = b;
    ///         b = c;
    ///     }
    ///     a
    /// }
    ///
    /// // # usage
    ///
    /// // this defines how the iterator must extend:
    /// // each node drawn from the iterator adds its children to the end of the iterator
    /// fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
    ///     &node.children
    /// }
    ///
    /// let mut rng = ChaCha8Rng::seed_from_u64(42);
    /// let roots = vec![
    ///     Node::new(50, &mut rng),
    ///     Node::new(20, &mut rng),
    ///     Node::new(40, &mut rng),
    /// ];
    ///
    /// let seq_sum: u64 = roots.iter().map(|x| x.seq_sum_fib()).sum();
    ///
    /// // A. exact length, recommended when possible
    ///
    /// let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();
    ///
    /// let sum = (&roots)
    ///     .into_par_rec_exact(extend, count)
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    ///
    /// // B. guide the computation with chunk size, when length is unknown
    ///
    /// let sum = (&roots)
    ///     .into_par_rec(extend)
    ///     .chunk_size(1024)
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    ///
    /// // C. eagerly convert to a flat iterator
    ///
    /// let sum = (&roots)
    ///     .into_par_rec(extend)
    ///     .into_eager()
    ///     .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
    ///     .sum();
    /// assert_eq!(sum, seq_sum);
    /// ```
    fn into_par_rec_exact<E, I>(
        self,
        extend: E,
        exact_len: usize,
    ) -> Par<ConcurrentRecursiveIterExact<Self::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&Self::Item) -> I + Sync;
}

impl<X> IntoParIterRecExact for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_rec_exact<E, I>(
        self,
        extend: E,
        exact_len: usize,
    ) -> Par<ConcurrentRecursiveIterExact<Self::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&Self::Item) -> I + Sync,
    {
        let con_rec_iter = ConcurrentRecursiveIterExact::new_exact(extend, self, exact_len);
        Par::new(DefaultRunner::default(), Params::default(), con_rec_iter)
    }
}
