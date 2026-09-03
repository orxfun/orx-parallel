use crate::collectables::par_extend::ParExtend;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;

impl<T: Ord + Send> ParExtend<T> for BinaryHeap<T> {
    type ThreadValues = Self;

    type OrderedThreadValues = Self;

    fn new_thread_values() -> Self::ThreadValues {
        Default::default()
    }

    fn new_ordered_thread_values() -> Self::OrderedThreadValues {
        Default::default()
    }

    // thread collect

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T) {
        collected.push(value);
    }

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>) {
        collected.extend(values)
    }

    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, _idx: usize, value: T) {
        Self::add_thread_value(collected, value);
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        _idx: usize,
        values: impl IntoIterator<Item = T>,
    ) {
        Self::add_thread_values(collected, values);
    }

    // opt: thread collect

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        _idx: usize,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()> {
        for value in values {
            collected.push(value?);
        }
        Some(())
    }

    // res: thread collect

    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        _idx: usize,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<(), E> {
        for value in values {
            collected.push(value?);
        }
        Ok(())
    }

    // add

    fn add_one(&mut self, value: T) {
        self.push(value);
    }

    // extend - merge

    fn extend_merge_infallibles(&mut self, results: Vec<Self::ThreadValues>) {
        for result in results {
            self.extend(result);
        }
    }

    fn extend_merge_ordered_infallibles(&mut self, results: Vec<Self::OrderedThreadValues>) {
        self.extend_merge_infallibles(results);
    }
}
