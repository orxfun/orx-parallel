use alloc::vec::Vec;

// TODO: document the trait
#[allow(missing_docs)]
pub trait ParExtend<T>: Extend<T> {
    type ThreadValues;

    type OrderedThreadValues;

    fn new_thread_values() -> Self::ThreadValues;

    fn new_ordered_thread_values() -> Self::OrderedThreadValues;

    // thread collect

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T);

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>);

    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, idx: usize, value: T);

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    );

    // opt: thread collect

    fn add_thread_optionals(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()> {
        for value in values {
            Self::add_thread_value(collected, value?);
        }
        Some(())
    }

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()>;

    // res: thread collect

    fn add_thread_fallibles<E>(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E> {
        for value in values {
            Self::add_thread_value(collected, value?)
        }
        Ok(())
    }

    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E>;

    // add

    fn add_one(&mut self, value: T);

    // extend

    fn extend_optionals(&mut self, optionals: impl IntoIterator<Item = Option<T>>) -> Option<()> {
        for value in optionals {
            self.add_one(value?);
        }
        Some(())
    }

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

    fn extend_merge_infallibles(&mut self, thread_results: Vec<Self::ThreadValues>);

    fn extend_merge_ordered_infallibles(&mut self, thread_results: Vec<Self::OrderedThreadValues>);

    fn extend_merge_optionals(
        &mut self,
        thread_results: Vec<Option<Self::ThreadValues>>,
    ) -> Option<()> {
        let infallibles: Option<Vec<Self::ThreadValues>> = thread_results.into_iter().collect();
        self.extend_merge_infallibles(infallibles?);
        Some(())
    }

    fn extend_merge_ordered_optionals(
        &mut self,
        thread_results: Vec<Option<Self::OrderedThreadValues>>,
    ) -> Option<()> {
        let infallibles: Option<Vec<Self::OrderedThreadValues>> =
            thread_results.into_iter().collect();
        self.extend_merge_ordered_infallibles(infallibles?);
        Some(())
    }

    fn extend_merge_fallibles<E>(
        &mut self,
        thread_results: Vec<Result<Self::ThreadValues, E>>,
    ) -> Result<(), E> {
        let infallibles: Result<Vec<Self::ThreadValues>, E> = thread_results.into_iter().collect();
        self.extend_merge_infallibles(infallibles?);
        Ok(())
    }

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
