use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::col_and_pos::ColAndPos;
use crate::collectables::par_extend_impl::idx_len::IdxLen;
use alloc::{vec, vec::Vec};
use core::hash::Hash;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
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

    fn extend_from_ordered_thread_results(&mut self, results: Vec<Self::OrderedThreadValues>) {
        todo!()
    }
}

// merge helpers

#[derive(Clone)]
struct ThLen {
    th: usize,
    len: usize,
}

impl ThLen {
    #[inline(always)]
    fn new(th: usize, len: usize) -> Self {
        Self { th, len }
    }
}
