use crate::collectables::par_extend_core::ParExtendCore;
use crate::collectables::par_extend_impl::utils::{ColAndPos, IdxLen, NextN};
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

impl<K: Ord + Send, V: Send> ParExtendCore<(K, V)> for BTreeMap<K, V> {
    type ThreadValues = Self;

    type OrderedThreadValues = ColAndPos<Self>;

    fn new_thread_values() -> Self::ThreadValues {
        Default::default()
    }

    fn new_ordered_thread_values() -> Self::OrderedThreadValues {
        Default::default()
    }

    // thread collect

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

    // opt: thread collect

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<(K, V)>>,
    ) -> Option<()> {
        let len_begin = collected.values.len();
        for value in values {
            let (key, value) = value?;
            _ = collected.values.insert(key, value);
        }

        let len = collected.values.len() - len_begin;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }

        Some(())
    }

    // res: thread collect

    fn add_ordered_thread_fallibles<E>(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Result<(K, V), E>>,
    ) -> Result<(), E> {
        let len_begin = collected.values.len();
        for value in values {
            let (key, value) = value?;
            collected.values.insert(key, value);
        }

        let len = collected.values.len() - len_begin;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }

        Ok(())
    }

    // add

    fn add_one(&mut self, (key, value): (K, V)) {
        _ = self.insert(key, value);
    }

    // extend - merge

    fn extend_merge_infallibles(&mut self, results: Vec<Self::ThreadValues>) {
        for result in results {
            self.extend(result);
        }
    }

    fn extend_merge_ordered_infallibles(&mut self, results: Vec<Self::OrderedThreadValues>) {
        let outer_len = results.len();
        let mut all_values = Vec::with_capacity(outer_len);
        let mut all_positions = Vec::with_capacity(outer_len);

        for x in results {
            all_values.push(x.values.into_iter());
            all_positions.push(x.positions);
        }

        let mut queue = BinaryHeap::with_capacity(outer_len);
        let mut pos_indices = vec![0; outer_len];
        for (th, positions) in all_positions.iter().enumerate() {
            if let Some(pos) = positions.first() {
                let node = ThLen::new(th, pos.len);
                queue.push(node, pos.idx);
            }
        }
        let mut curr_t = queue.pop_node();

        while let Some(ThLen { th, len }) = curr_t {
            let chunk = NextN::new(&mut all_values[th], len);
            self.extend(chunk);

            pos_indices[th] += 1;
            curr_t = match all_positions[th].get(pos_indices[th]) {
                Some(pos) => {
                    let node = ThLen::new(th, pos.len);
                    Some(queue.push_then_pop(node, pos.idx).0)
                }
                None => queue.pop_node(),
            };
        }
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
