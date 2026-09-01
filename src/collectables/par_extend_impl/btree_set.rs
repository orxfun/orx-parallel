use crate::collectables::par_extend::ParExtend;
use alloc::collections::BTreeSet;
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

impl<T: Ord> ParExtend<T> for BTreeSet<T> {
    type ThreadValues = Self;

    type OrderedThreadValues = SetAndPositions<T>;

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T) {
        _ = collected.insert(value);
    }

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>) {
        collected.extend(values)
    }

    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, idx: usize, value: T) {
        let inserted = collected.values.insert(value);
        if inserted {
            collected.positions.push(IdxLen { idx, len: 1 });
        }
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    ) {
        let len_before = collected.values.len();
        collected.values.extend(values);

        let len = collected.values.len() - len_before;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }
    }

    // extend

    fn extend_from_ordered_thread_results(&mut self, results: Vec<Self::OrderedThreadValues>) {
        let outer_len = results.len();
        let mut all_values = Vec::with_capacity(outer_len);
        let mut all_positions = Vec::with_capacity(outer_len);

        for x in results {
            all_values.push(x.values.into_iter());
            all_positions.push(x.positions);
        }

        let mut queue = BinaryHeap::with_capacity(outer_len);
        let mut pos_indices = vec![0; outer_len];
        for (v, positions) in all_positions.iter().enumerate() {
            if let Some(pos) = positions.first() {
                queue.push(ThAndLen::new(v, pos.len), pos.idx);
            }
        }
        let mut curr_t = queue.pop_node();

        while let Some(ThAndLen { t, len }) = curr_t {
            for _ in 0..len {
                let value = all_values[t].next();
                // TODO: add safety note
                let value = unsafe { value.unwrap_unchecked() };
                _ = self.insert(value);
            }

            pos_indices[t] += 1;
            curr_t = match all_positions[t].get(pos_indices[t]) {
                Some(pos) => Some(queue.push_then_pop(ThAndLen::new(t, pos.len), pos.idx).0),
                None => queue.pop_node(),
            };
        }
    }
}

struct IdxLen {
    idx: usize,
    len: usize,
}

pub struct SetAndPositions<T> {
    values: BTreeSet<T>,
    positions: Vec<IdxLen>,
}

impl<T> Default for SetAndPositions<T> {
    fn default() -> Self {
        Self {
            values: BTreeSet::new(),
            positions: Vec::new(),
        }
    }
}

// merge helpers

#[derive(Clone)]
struct ThAndLen {
    t: usize,
    len: usize,
}

impl ThAndLen {
    #[inline(always)]
    fn new(t: usize, len: usize) -> Self {
        Self { t, len }
    }
}
