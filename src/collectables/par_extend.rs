use alloc::vec::Vec;

/// Parallel collection support for destinations that must be populated from multiple worker threads.
///
/// `ParExtend` extends the standard `Extend` contract with the information needed to collect
/// values produced concurrently and merge them back into a single destination. This is the
/// abstraction used by the parallel iterators to assemble results without depending on a
/// specific collection type or on a legacy common trait implementation.
///
/// A destination can support either arbitrary-order collection or ordered collection. In the
/// first case, each thread accumulates a local buffer and the final merge combines those buffers
/// into the destination. In the second case, each thread keeps index-aware entries so the final
/// merge can restore the original ordering.
///
/// This is intended for parallel collection from fallible, optional, and infallible item streams,
/// while preserving the corresponding short-circuit semantics and merge behavior.
pub trait ParExtend<T>: Extend<T> {
    /// Per-thread accumulation buffer for arbitrary-order collection.
    type ThreadValues: Send;

    /// Per-thread accumulation buffer for ordered collection, where the position of each emitted
    /// item is known by its original index.
    type OrderedThreadValues: Send;

    /// Creates an empty buffer for a single worker thread in arbitrary-order collection mode.
    fn new_thread_values() -> Self::ThreadValues;

    /// Creates an empty buffer for a single worker thread in ordered collection mode.
    fn new_ordered_thread_values() -> Self::OrderedThreadValues;

    // thread collect

    /// Adds a single value into a thread-local arbitrary-order buffer.
    fn add_thread_value(collected: &mut Self::ThreadValues, value: T);

    /// Adds all values from an iterator into a thread-local arbitrary-order buffer.
    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>);

    /// Adds a single value at the given index to a thread-local ordered buffer.
    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, idx: usize, value: T);

    /// Adds all values from an iterator into a thread-local ordered buffer, assigning the original
    /// indices to each entry.
    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    );

    // opt: thread collect

    /// Consumes an iterator of optional values and stores only the `Some` items in a thread-local
    /// arbitrary-order buffer.
    fn add_thread_optionals(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()> {
        for value in values {
            Self::add_thread_value(collected, value?);
        }
        Some(())
    }

    /// Consumes an iterator of optional values and stores only the `Some` items in a thread-local
    /// ordered buffer.
    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()>;

    // res: thread collect

    /// Consumes an iterator of fallible values and stores only the successful items in a
    /// thread-local arbitrary-order buffer.
    fn add_thread_fallibles<E>(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E> {
        for value in values {
            Self::add_thread_value(collected, value?)
        }
        Ok(())
    }

    /// Consumes an iterator of fallible values and stores only the successful items in a
    /// thread-local ordered buffer.
    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E>;

    // add

    /// Inserts a single value into the destination collection.
    fn add_one(&mut self, value: T);

    // extend

    /// Extends the destination with optional items, stopping early if an `Option::None` is
    /// encountered while preserving the short-circuit semantics used by the parallel APIs.
    fn extend_optionals(&mut self, optionals: impl IntoIterator<Item = Option<T>>) -> Option<()> {
        for value in optionals {
            self.add_one(value?);
        }
        Some(())
    }

    /// Extends the destination with fallible items, propagating the first error encountered.
    fn extend_fallibles<E>(
        &mut self,
        fallibles: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E> {
        for value in fallibles {
            self.add_one(value?);
        }
        Ok(())
    }

    // extend - merge

    /// Merges thread-local arbitrary-order results into the destination collection.
    fn extend_merge_infallibles(&mut self, thread_results: Vec<Self::ThreadValues>);

    /// Merges thread-local ordered results into the destination collection, restoring the original
    /// item order before the final collection is completed.
    fn extend_merge_ordered_infallibles(&mut self, thread_results: Vec<Self::OrderedThreadValues>);

    /// Merges arbitrary-order optional thread results, returning `None` if any thread reported a
    /// stop condition.
    fn extend_merge_optionals(
        &mut self,
        thread_results: Vec<Option<Self::ThreadValues>>,
    ) -> Option<()> {
        let infallibles: Option<Vec<Self::ThreadValues>> = thread_results.into_iter().collect();
        self.extend_merge_infallibles(infallibles?);
        Some(())
    }

    /// Merges ordered optional thread results, returning `None` if any thread reported a stop
    /// condition.
    fn extend_merge_ordered_optionals(
        &mut self,
        thread_results: Vec<Option<Self::OrderedThreadValues>>,
    ) -> Option<()> {
        let infallibles: Option<Vec<Self::OrderedThreadValues>> =
            thread_results.into_iter().collect();
        self.extend_merge_ordered_infallibles(infallibles?);
        Some(())
    }

    /// Merges arbitrary-order fallible thread results, propagating the first error encountered.
    fn extend_merge_fallibles<E>(
        &mut self,
        thread_results: Vec<Result<Self::ThreadValues, E>>,
    ) -> Result<(), E> {
        let infallibles: Result<Vec<Self::ThreadValues>, E> = thread_results.into_iter().collect();
        self.extend_merge_infallibles(infallibles?);
        Ok(())
    }

    /// Merges ordered fallible thread results, propagating the first error encountered while also
    /// restoring the original index ordering.
    fn extend_merge_ordered_fallibles<E>(
        &mut self,
        thread_results: Vec<Result<Self::OrderedThreadValues, E>>,
    ) -> Result<(), E> {
        let infallibles: Result<Vec<Self::OrderedThreadValues>, E> =
            thread_results.into_iter().collect();
        self.extend_merge_ordered_infallibles(infallibles?);
        Ok(())
    }
}
