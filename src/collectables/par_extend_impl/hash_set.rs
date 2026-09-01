use crate::collectables::par_extend::ParExtend;
use alloc::vec::Vec;
use core::hash::Hash;
use std::collections::HashSet;

impl<T: Hash + Eq> ParExtend<T> for HashSet<T> {
    type ThreadValues = Self;

    type OrderedThreadValues = Self;

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T) {
        _ = collected.insert(value);
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

    // extend

    fn extend_from_thread_results(&mut self, results: Vec<Self::ThreadValues>) {
        for result in results {
            self.extend(result);
        }
    }

    fn extend_from_ordered_thread_results(&mut self, results: Vec<Self::OrderedThreadValues>) {
        self.extend_from_thread_results(results);
    }
}
