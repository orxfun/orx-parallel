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
        let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
        let initial_len = self.len();
        let total_len = initial_len + collected_len;

        // let mut queue = BinaryHeap::with_capacity(results.len());
        let mut pos_indices = vec![0; results.len()];
        // for (v, vec) in results.iter().enumerate() {
        //     if let Some(pos) = vec.positions.first() {
        //         queue.push(VecPos::new(v, 0, pos.len), pos.idx);
        //     }
        // }
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
