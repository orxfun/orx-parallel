use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::col_and_pos::ColAndPos;
use crate::collectables::par_extend_impl::idx_len::IdxLen;
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

impl<K: Ord, V> ParExtend<(K, V)> for BTreeMap<K, V> {
    type ThreadValues = Self;

    type OrderedThreadValues = ColAndPos<Self>;

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
        (key, value): (K, V),
    ) {
        let inserted = collected.values.insert(key, value).is_none();
        if inserted {
            collected.positions.push(IdxLen { idx, len: 1 });
        }
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = (K, V)>,
    ) {
        let len_before = collected.values.len();
        collected.values.extend(values);

        let len = collected.values.len() - len_before;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }
    }

    fn extend_from_ordered_thread_results(
        &mut self,
        thread_results: Vec<Self::OrderedThreadValues>,
    ) {
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
