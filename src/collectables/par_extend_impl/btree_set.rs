use crate::collectables::par_extend::ParExtend;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

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
        for value in values {
            _ = collected.values.insert(value);
        }
        let len = collected.values.len() - len_before;
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
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
