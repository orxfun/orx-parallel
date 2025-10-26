use crate::{DefaultRunner, Params, computational_variants::Par};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

// unknown size

/// Trait to convert an iterator into a recursive parallel iterator together with the `extend` method.
/// Recursive iterators are most useful for defining parallel computations over non-linear data structures
/// such as trees or graphs.
///
/// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
///
/// It is recursive due to the extension. The recursive parallel iterator will yield
/// * all initial elements contained in this iterator,
/// * all elements dynamically added to the queue with the `extend` method while processing the elements.
///
/// You may read more about the [`ConcurrentRecursiveIter`].
///
/// [`ParIter`]: crate::ParIter
pub trait IntoParIterRec
where
    Self: IntoIterator,
    Self::Item: Send,
{
    /// Converts this iterator into a recursive parallel iterator together with the `extend` method.
    /// Recursive iterators are most useful for defining parallel computations over non-linear data structures
    /// such as trees or graphs.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements dynamically added to the queue with the `extend` method while processing the elements.
    ///
    /// You may read more about the [`ConcurrentRecursiveIter`].
    ///
    /// The `extend` function defines the recursive expansion behavior. It takes two arguments:
    /// * `element: &Self::Item` is the item being processed.
    /// * `queue: Queue<Self::Item, P>` is the queue of remaining elements/tasks which exposes two methods:
    ///   * `push(item)` allows us to add one item to the queue,
    ///   * `extend(items)` allows us to add all of the items to the queue. Here `items` must have a known
    ///     size (`ExactSizeIterator`).
    ///
    /// Adding children one-by-one with `push` or all together with `extend` might be the extreme options.
    /// Actually, any intermediate approach is also possible. For instance, we can choose to `extend` in
    /// chunks of say 50 tasks. If the item happens to create 140 children, we can handle this with four
    /// `extend` calls.
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
    /// The decision is use-case specific and best to benchmark for the specific input.
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
    /// We can use `initial_elements.into_par_rec_exact(extend, count)` to create the iterator with exact length.
    ///
    /// ## B. Recursive Iterator with Unknown Length
    ///
    /// If we cannot know or it is expensive to know the exact length of the iterator ahead of time, we can
    /// still create a recursive parallel iterator. In these cases; however, it is recommended to provide
    /// chunk size explicitly depending on the number of threads that will be used and any estimate on the exact
    /// length.
    ///
    /// We can use `initial_elements.into_par_rec(extend)` to create the iterator without length information.
    ///
    /// ## C. Into Eager Transformation
    ///
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks using [`into_eager`] transformation.
    ///
    /// We can use `initial_elements.into_par_rec(extend).into_eager()` to create the flattened iterator.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIter`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIter
    /// [`into_eager`]: crate::computational_variants::Par::into_eager
    ///
    /// ## Examples
    ///
    /// In the following example we perform some parallel computations over a tree.
    /// It demonstrates that a "recursive parallel iterator" is just a parallel iterator with
    /// access to all [`ParIter`] methods.
    /// Once we create the recursive parallel iterator with the `extend` definition, we can use it as
    /// a regular parallel iterator.
    ///
    /// Unfortunately, the example requires a long set up for completeness. Note that the relevant
    /// code blocks begin after line `// parallel reduction`.
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha8Rng;
    /// use std::{collections::HashSet, ops::Range};
    ///
    /// pub struct Node<T> {
    ///     pub idx: usize,
    ///     pub data: T,
    ///     pub children: Vec<Node<T>>,
    /// }
    ///
    /// impl<T> Node<T> {
    ///     fn create_node(out_edges: &[Vec<usize>], idx: usize, data: fn(usize) -> T) -> Node<T> {
    ///         Node {
    ///             idx,
    ///             data: data(idx),
    ///             children: out_edges[idx]
    ///                 .iter()
    ///                 .map(|child_idx| Self::create_node(out_edges, *child_idx, data))
    ///                 .collect(),
    ///         }
    ///     }
    ///
    ///     pub fn new_tree(
    ///         num_nodes: usize,
    ///         degree: Range<usize>,
    ///         data: fn(usize) -> T,
    ///         rng: &mut impl Rng,
    ///     ) -> Node<T> {
    ///         assert!(num_nodes >= 2);
    ///
    ///         let mut leaves = vec![0];
    ///         let mut remaining: Vec<_> = (1..num_nodes).collect();
    ///         let mut edges = vec![];
    ///         let mut out_edges = vec![vec![]; num_nodes];
    ///
    ///         while !remaining.is_empty() {
    ///             let leaf_idx = rng.random_range(0..leaves.len());
    ///             let leaf = leaves.remove(leaf_idx);
    ///
    ///             let degree = rng.random_range(degree.clone());
    ///             match degree == 0 {
    ///                 true => leaves.push(leaf),
    ///                 false => {
    ///                     let children_indices: HashSet<_> = (0..degree)
    ///                         .map(|_| rng.random_range(0..remaining.len()))
    ///                         .collect();
    ///
    ///                     let mut sorted: Vec<_> = children_indices.iter().copied().collect();
    ///                     sorted.sort();
    ///
    ///                     edges.extend(children_indices.iter().map(|c| (leaf, remaining[*c])));
    ///                     out_edges[leaf] = children_indices.iter().map(|c| remaining[*c]).collect();
    ///                     leaves.extend(children_indices.iter().map(|c| remaining[*c]));
    ///
    ///                     for idx in sorted.into_iter().rev() {
    ///                         remaining.remove(idx);
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///
    ///         Self::create_node(&out_edges, 0, data)
    ///     }
    /// }
    ///
    /// let num_nodes = 1_000;
    /// let out_degree = 0..100;
    /// let mut rng = ChaCha8Rng::seed_from_u64(42);
    /// let data = |idx: usize| idx.to_string();
    /// let root = Node::new_tree(num_nodes, out_degree, data, &mut rng);
    ///
    /// let compute = |node: &Node<String>| node.data.parse::<u64>().unwrap();
    ///
    /// // parallel reduction
    ///
    /// fn extend<'a, T: Sync>(node: &&'a Node<T>, queue: &Queue<&'a Node<T>>) {
    ///     queue.extend(&node.children);
    /// }
    ///
    /// let sum = [&root].into_par_rec(extend).map(compute).sum();
    /// assert_eq!(sum, 499500);
    ///
    /// // or any parallel computation such as map->filter->collect
    ///
    /// let result: Vec<_> = [&root]
    ///     .into_par_rec(extend)
    ///     .map(compute)
    ///     .filter(|x| x.is_multiple_of(7))
    ///     .collect();
    /// assert_eq!(result.len(), 143);
    ///
    /// // or filter during extension
    /// fn extend_filtered<'a>(node: &&'a Node<String>, queue: &Queue<&'a Node<String>>) {
    ///     for child in &node.children {
    ///         if child.idx != 42 {
    ///             queue.push(child);
    ///         }
    ///     }
    /// }
    ///
    /// let sum = [&root].into_par_rec(extend_filtered).map(compute).sum();
    /// assert_eq!(sum, 499458);
    /// ```
    fn into_par_rec<E>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E>, DefaultRunner>
    where
        E: Fn(&Self::Item, &Queue<Self::Item>) + Sync;

    /// Converts this iterator into a recursive parallel iterator together with the `extend` method.
    /// Recursive iterators are most useful for defining parallel computations over non-linear data structures
    /// such as trees or graphs.
    ///
    /// Created parallel iterator is a regular parallel iterator; i.e., we have access to all [`ParIter`] features.
    ///
    /// It is recursive due to the extension. The recursive parallel iterator will yield
    /// * all initial elements contained in this iterator,
    /// * all elements dynamically added to the queue with the `extend` method while processing the elements.
    ///
    /// You may read more about the [`ConcurrentRecursiveIter`].
    ///
    /// The `extend` function defines the recursive expansion behavior. It takes two arguments:
    /// * `element: &Self::Item` is the item being processed.
    /// * `queue: Queue<Self::Item, P>` is the queue of remaining elements/tasks which exposes two methods:
    ///   * `push(item)` allows us to add one item to the queue,
    ///   * `extend(items)` allows us to add all of the items to the queue. Here `items` must have a known
    ///     size (`ExactSizeIterator`).
    ///
    /// Adding children one-by-one with `push` or all together with `extend` might be the extreme options.
    /// Actually, any intermediate approach is also possible. For instance, we can choose to `extend` in
    /// chunks of say 50 tasks. If the item happens to create 140 children, we can handle this with four
    /// `extend` calls.
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
    /// The decision is use-case specific and best to benchmark for the specific input.
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
    /// We can use `initial_elements.into_par_rec_exact(extend, count)` to create the iterator with exact length.
    ///
    /// ## B. Recursive Iterator with Unknown Length
    ///
    /// If we cannot know or it is expensive to know the exact length of the iterator ahead of time, we can
    /// still create a recursive parallel iterator. In these cases; however, it is recommended to provide
    /// chunk size explicitly depending on the number of threads that will be used and any estimate on the exact
    /// length.
    ///
    /// We can use `initial_elements.into_par_rec(extend)` to create the iterator without length information.
    ///
    /// ## C. Into Eager Transformation
    ///
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks using [`into_eager`] transformation.
    ///
    /// We can use `initial_elements.into_par_rec(extend).into_eager()` to create the flattened iterator.
    ///
    /// [`ParIter`]: crate::ParIter
    /// [`ConcurrentRecursiveIter`]: orx_concurrent_recursive_iter::ConcurrentRecursiveIter
    /// [`into_eager`]: crate::computational_variants::Par::into_eager
    ///
    /// ## Examples
    ///
    /// In the following example we perform some parallel computations over a tree.
    /// It demonstrates that a "recursive parallel iterator" is just a parallel iterator with
    /// access to all [`ParIter`] methods.
    /// Once we create the recursive parallel iterator with the `extend` definition, we can use it as
    /// a regular parallel iterator.
    ///
    /// Unfortunately, the example requires a long set up for completeness. Note that the relevant
    /// code blocks begin after line `// parallel reduction`.
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha8Rng;
    /// use std::{collections::HashSet, ops::Range};
    ///
    /// pub struct Node<T> {
    ///     pub idx: usize,
    ///     pub data: T,
    ///     pub children: Vec<Node<T>>,
    /// }
    ///
    /// impl<T> Node<T> {
    ///     fn create_node(out_edges: &[Vec<usize>], idx: usize, data: fn(usize) -> T) -> Node<T> {
    ///         Node {
    ///             idx,
    ///             data: data(idx),
    ///             children: out_edges[idx]
    ///                 .iter()
    ///                 .map(|child_idx| Self::create_node(out_edges, *child_idx, data))
    ///                 .collect(),
    ///         }
    ///     }
    ///
    ///     pub fn new_tree(
    ///         num_nodes: usize,
    ///         degree: Range<usize>,
    ///         data: fn(usize) -> T,
    ///         rng: &mut impl Rng,
    ///     ) -> Node<T> {
    ///         assert!(num_nodes >= 2);
    ///
    ///         let mut leaves = vec![0];
    ///         let mut remaining: Vec<_> = (1..num_nodes).collect();
    ///         let mut edges = vec![];
    ///         let mut out_edges = vec![vec![]; num_nodes];
    ///
    ///         while !remaining.is_empty() {
    ///             let leaf_idx = rng.random_range(0..leaves.len());
    ///             let leaf = leaves.remove(leaf_idx);
    ///
    ///             let degree = rng.random_range(degree.clone());
    ///             match degree == 0 {
    ///                 true => leaves.push(leaf),
    ///                 false => {
    ///                     let children_indices: HashSet<_> = (0..degree)
    ///                         .map(|_| rng.random_range(0..remaining.len()))
    ///                         .collect();
    ///
    ///                     let mut sorted: Vec<_> = children_indices.iter().copied().collect();
    ///                     sorted.sort();
    ///
    ///                     edges.extend(children_indices.iter().map(|c| (leaf, remaining[*c])));
    ///                     out_edges[leaf] = children_indices.iter().map(|c| remaining[*c]).collect();
    ///                     leaves.extend(children_indices.iter().map(|c| remaining[*c]));
    ///
    ///                     for idx in sorted.into_iter().rev() {
    ///                         remaining.remove(idx);
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///
    ///         Self::create_node(&out_edges, 0, data)
    ///     }
    /// }
    ///
    /// let num_nodes = 1_000;
    /// let out_degree = 0..100;
    /// let mut rng = ChaCha8Rng::seed_from_u64(42);
    /// let data = |idx: usize| idx.to_string();
    /// let root = Node::new_tree(num_nodes, out_degree, data, &mut rng);
    ///
    /// let compute = |node: &Node<String>| node.data.parse::<u64>().unwrap();
    ///
    /// // parallel reduction
    ///
    /// fn extend<'a, T: Sync>(node: &&'a Node<T>, queue: &Queue<&'a Node<T>>) {
    ///     queue.extend(&node.children);
    /// }
    ///
    /// let sum = [&root].into_par_rec(extend).map(compute).sum();
    /// assert_eq!(sum, 499500);
    ///
    /// // or any parallel computation such as map->filter->collect
    ///
    /// let result: Vec<_> = [&root]
    ///     .into_par_rec(extend)
    ///     .map(compute)
    ///     .filter(|x| x.is_multiple_of(7))
    ///     .collect();
    /// assert_eq!(result.len(), 143);
    ///
    /// // or filter during extension
    /// fn extend_filtered<'a>(node: &&'a Node<String>, queue: &Queue<&'a Node<String>>) {
    ///     for child in &node.children {
    ///         if child.idx != 42 {
    ///             queue.push(child);
    ///         }
    ///     }
    /// }
    ///
    /// let sum = [&root].into_par_rec(extend_filtered).map(compute).sum();
    /// assert_eq!(sum, 499458);
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
