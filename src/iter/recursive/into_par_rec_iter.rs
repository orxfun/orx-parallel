use crate::{DefaultRunner, Params, computational_variants::Par};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

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
    /// Converts this iterator into a recursive parallel iterator together with the `extend` method.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements dynamically added to the queue with the `extend` method while processing the elements.
    ///
    /// You may read more about the [`ConcurrentRecursiveIter`].
    ///
    /// # Examples
    ///
    /// The following example has some code to set up until the `# usage` line. Notice that the `Node`
    /// is a non-linear data structure, each node having children nodes to be recursively processed.
    ///
    /// We have three initial elements `roots`.
    ///
    /// We want to compute sum of Fibonacci numbers of values of all nodes descending from the roots.
    ///
    /// The `expand` function defines the recursive expansion behavior. It takes two arguments:
    /// * `element: &Self::Item` is the item being processed.
    /// * `queue: Queue<Self::Item, P>` is the queue of remaining elements/tasks which exposes two methods:
    ///   * `push(item)` allows us to add one item to the queue,
    ///   * `extend(items)` allows us to add all of the items to the queue. Here `items` must have a known
    ///     size (`ExactSizeIterator`).
    ///
    /// Using either of the methods might be beneficial for different use cases.
    ///
    /// Pushing children one by one makes the new task available for other threads as fast as possible. Further,
    /// when we don't know the exact number of children ahead of time, and we don't want to use heap allocation
    /// to store the children in a vec before adding them to the queue just to make it sized, we can add the
    /// elements one-by-one with the `queue.push(item)` method. On the other hand, this approach will have more
    /// parallelization overhead.
    ///
    /// When we extending children all at once using `queue.extend(items)`, we minimize the parallelization overhead
    /// for adding tasks to the queue. On the other hand, the children will be available only when writing of all
    /// children to the queue is complete which might cause idleness when tasks are scarce. Still, the recommendation
    /// is to try to `extend` first whenever possible due to the following: (i) if we extend with a lot of children,
    /// the tasks will not be scarce; (ii) and if we extend with only a few of items, the delay of making the tasks
    /// available for other threads will be short.
    ///
    /// Nevertheless, the decision is use-case specific and best to benchmark for the specific input.
    ///
    /// This crate makes use of the [`ConcurrentRecursiveIter`] for this computation and provides three ways to execute
    /// this computation in parallel.
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
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// Note that exact size will be obtained during flattening; and hence, we do not need to provide the `count`.
    ///
    /// In the example, we create eagerly flattened parallel iterator with the `(&roots).into_par_rec(extend).into_eager()` call.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIter`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIter
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
    /// fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
    ///     queue.extend(&node.children);
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
    fn into_par_rec<E>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E>, DefaultRunner>
    where
        E: Fn(&Self::Item, &Queue<Self::Item>) + Sync;

    /// Converts this iterator into a recursive parallel iterator together with the `extend` method.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements dynamically added to the queue with the `extend` method while processing the elements.
    ///
    /// You may read more about the [`ConcurrentRecursiveIter`].
    ///
    /// # Examples
    ///
    /// The following example has some code to set up until the `# usage` line. Notice that the `Node`
    /// is a non-linear data structure, each node having children nodes to be recursively processed.
    ///
    /// We have three initial elements `roots`.
    ///
    /// We want to compute sum of Fibonacci numbers of values of all nodes descending from the roots.
    ///
    /// The `expand` function defines the recursive expansion behavior. It takes two arguments:
    /// * `element: &Self::Item` is the item being processed.
    /// * `queue: Queue<Self::Item, P>` is the queue of remaining elements/tasks which exposes two methods:
    ///   * `push(item)` allows us to add one item to the queue,
    ///   * `extend(items)` allows us to add all of the items to the queue. Here `items` must have a known
    ///     size (`ExactSizeIterator`).
    ///
    /// Using either of the methods might be beneficial for different use cases.
    ///
    /// Pushing children one by one makes the new task available for other threads as fast as possible. Further,
    /// when we don't know the exact number of children ahead of time, and we don't want to use heap allocation
    /// to store the children in a vec before adding them to the queue just to make it sized, we can add the
    /// elements one-by-one with the `queue.push(item)` method. On the other hand, this approach will have more
    /// parallelization overhead.
    ///
    /// When we extending children all at once using `queue.extend(items)`, we minimize the parallelization overhead
    /// for adding tasks to the queue. On the other hand, the children will be available only when writing of all
    /// children to the queue is complete which might cause idleness when tasks are scarce. Still, the recommendation
    /// is to try to `extend` first whenever possible due to the following: (i) if we extend with a lot of children,
    /// the tasks will not be scarce; (ii) and if we extend with only a few of items, the delay of making the tasks
    /// available for other threads will be short.
    ///
    /// Nevertheless, the decision is use-case specific and best to benchmark for the specific input.
    ///
    /// This crate makes use of the [`ConcurrentRecursiveIter`] for this computation and provides three ways to execute
    /// this computation in parallel.
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
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// Note that exact size will be obtained during flattening; and hence, we do not need to provide the `count`.
    ///
    /// In the example, we create eagerly flattened parallel iterator with the `(&roots).into_par_rec(extend).into_eager()` call.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIter`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIter
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
    /// fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
    ///     queue.extend(&node.children);
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
    fn into_par_rec_exact<E>(
        self,
        extend: E,
        exact_len: usize,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E>, DefaultRunner>
    where
        E: Fn(&Self::Item, &Queue<Self::Item>) + Sync;
}

impl<X> IntoParIterRec for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_rec<E>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E>, DefaultRunner>
    where
        E: Fn(&Self::Item, &Queue<Self::Item>) + Sync,
    {
        let con_rec_iter = ConcurrentRecursiveIter::new(self, extend);
        Par::new(DefaultRunner::default(), Params::default(), con_rec_iter)
    }

    fn into_par_rec_exact<E>(
        self,
        extend: E,
        exact_len: usize,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E>, DefaultRunner>
    where
        E: Fn(&Self::Item, &Queue<Self::Item>) + Sync,
    {
        let con_rec_iter = ConcurrentRecursiveIter::new_exact(self, extend, exact_len);
        Par::new(DefaultRunner::default(), Params::default(), con_rec_iter)
    }
}
