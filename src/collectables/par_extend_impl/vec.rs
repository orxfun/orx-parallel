use crate::collectables::par_extend::{Contiguous, ParExtend};
use alloc::vec::Vec;

impl<T> ParExtend<T> for Vec<T> {
    type ThreadValues = Self;

    type OrderedThreadValues = VecAndPositions<T>;

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T) {
        collected.push(value);
    }

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>) {
        collected.extend(values)
    }

    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, idx: usize, value: T) {
        collected.values.push(value);
        collected.positions.push(IdxLen { idx, len: 1 });
    }

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    ) {
        let len_begin = collected.values.len();
        collected.values.extend(values);

        let len = collected.values.len() - len_begin;
        collected.positions.push(IdxLen { idx, len });
    }
}

impl<T> Contiguous<T> for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }

    unsafe fn ptr(&mut self, idx: usize) -> *mut T {
        debug_assert!(idx < self.len(), "index out of bounds");
        let p = self.as_mut_ptr();
        unsafe { p.add(idx) }
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity(), "setting len beyond capacity");
        unsafe { Vec::set_len(self, new_len) };
    }
}

// ordered thread values

struct IdxLen {
    idx: usize,
    len: usize,
}

pub struct VecAndPositions<T> {
    values: Vec<T>,
    positions: Vec<IdxLen>,
}
