# orx-parallel

## Easy to Configure

Complexity of distribution of work to parallel threads is boiled down to two straightforward parameters which are easy to reason about:
* [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.NumThreads.html) represents the degree of parallelization. It can be set to one of the two variants:
  * `Auto`: All threads will be assumed to be available. This is an upper bound; whenever the computation is not sufficiently challenging, this number may not be reached.
  * `Max(n)`: The computation can spawn at most n threads. NumThreads::Max(1) is equivalent to sequential execution.
* [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This parameter aims to balance the overhead of parallelization and cost of heterogeneity of tasks. It can be set to one of the three variants:
  * `Auto`: The library aims to select the best value in order to minimize computation time.
  * `Exact(c)`: Chunk sizes will be c. This variant gives the control completely to the caller, and hence, suits best to computations to be tuned.
  * `Min(c)`: Chunk sizes will be at least c. However, the execution is allowed to pull more elements depending on characteristics of the inputs and used number of threads in order to reduce the impact of parallelization overhead.

Having control on these two parameters and being able to configure each computation easily and individually is useful in various ways.

### Better Strategy by Problem Knowledge

Both number of threads and chunk size have `Auto` settings which perform well in general. However, there exists no strategy that performs best for all computations or input characteristics. Sometimes we know the best configuration for our problem.

For instance, consider a problem where we want to `find` the first feasible or acceptable solution, where each trial is computationally expensive. Of course, we want the computation to terminate as soon as possible once a solution is found.
* Setting chunk size to 1 allows for the quickest termination once the solution is found. However, it would have the greatest parallelization overhead.
* Setting a sufficiently large chunk size would have much lower parallelization overhead. However, once a thread finds a solution, the other threads would still need to complete their chunk before termination.

In this scenario where the computation is challenging, parallelization overhead is negligible. Therefore, we could simply set the chunk size to one.  A similar example is constructed in [benches/map_find_expensive.rs](https://github.com/orxfun/orx-parallel/blob/main/benches/map_find_expensive.rs), where this trivial strategy indeed turns out to be optimal.
* `Auto` chunk size settings, as well as, `rayon`'s default settings find the solution in slightly longer time than the sequential computation. This is an example case where parallelization can backfire.
* On the other hand, simply setting the chunk size to 1, we observe that `Par` finds the solution ~15 times faster than the sequential.

### Better Strategy by Tuning

A well known concern in parallelization is the diminishing returns from added resources. Doubling the number of threads usually does not halve the computation time. We usually seek a *sweet spot* where the gain is justified by the allocated resources.

Being able to have complete control on these parameters allows to benchmark and tune performance-critical computations for the relevant inputs on target platforms.

### Parallelization in a Concurrent Application

Consider the following two extreme strategies for an api responding to cpu-heavy computation requests:

* **sequential**: We can handle each request sequentially. Since the api will serve multiple requests concurrently, we could still utilize available resources. However, we could be under-utilizing in low-traffic time intervals. Further, we might not be achieving desired average response time per request.
* **all-in**: We might allow each request to utilize all available resources when it arrives. Under low-traffic, this could minimize the response time. However, recall the diminishing improvements of additional computing resources allocated to one computation. Although we are keeping resources busy, we could be utilizing them very inefficiently and we could perform worse than the sequential strategy in high traffic periods.

A simple strategy could be to limit maximum number of threads to be used per computation. The limit can be a sweet spot discovered empirically. This could already help in preventing the extreme problems discussed above.

A more sophisticated approach could be to implement a resource allocator which dynamically decides on the number of threads of a request depending on the traffic and utilization of resources at the instant the request arrives.
