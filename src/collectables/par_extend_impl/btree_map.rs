use crate::collectables::par_extend::ParExtend;
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

impl<K: Ord, V> ParExtend<(K, V)> for BTreeMap<K, V> {
    type ThreadValues = Self;

    type OrderedThreadValues = usize;

    fn add_thread_value(collected: &mut Self::ThreadValues, (key, value): (K, V)) {
        _ = collected.insert(key, value);
    }

    fn add_thread_values(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = (K, V)>,
    ) {
        collected.extend(values)
    }

    fn add_ordered_thread_value(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        value: (K, V),
    ) {
        todo!()
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = (K, V)>,
    ) {
        todo!()
    }

    fn extend_from_ordered_thread_results(
        &mut self,
        thread_results: Vec<Self::OrderedThreadValues>,
    ) {
        todo!()
    }
}
