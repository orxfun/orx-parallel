use crate::collectables::par_extend::ParExtend;
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

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
        if len > 0 {
            collected.positions.push(IdxLen { idx, len });
        }
    }

    fn extend_from_ordered_thread_results(&mut self, mut results: Vec<Self::OrderedThreadValues>) {
        let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
        self.reserve(collected_len);
        let initial_len = self.len();
        let total_len = initial_len + collected_len;

        let mut queue = BinaryHeap::with_capacity(results.len());
        let mut pos_indices = vec![0; results.len()];

        for (v, vec) in results.iter().enumerate() {
            if let Some(pos) = vec.positions.first() {
                queue.push(VecPos::new(v, 0, pos.len), pos.idx);
            }
        }
        let mut curr_v = queue.pop_node();
        let mut ptr_dst = unsafe { self.as_mut_ptr().add(initial_len) };

        while let Some(VecPos { v, beg, len }) = curr_v {
            let ptr_src = unsafe { results[v].values.as_ptr().add(beg) };
            unsafe { ptr_dst.copy_from_nonoverlapping(ptr_src, len) };

            pos_indices[v] += 1;
            curr_v = match results[v].positions.get(pos_indices[v]) {
                Some(pos) => {
                    let beg = beg + len;
                    Some(queue.push_then_pop(VecPos::new(v, beg, pos.len), pos.idx).0)
                }
                None => queue.pop_node(),
            };

            ptr_dst = unsafe { ptr_dst.add(len) };
        }

        for vec in results.iter_mut() {
            // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
            // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
            unsafe { vec.values.set_len(0) };
        }

        unsafe { self.set_len(total_len) };
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

impl<T> Default for VecAndPositions<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            positions: Vec::new(),
        }
    }
}

#[derive(Clone)]
struct VecPos {
    v: usize,
    beg: usize,
    len: usize,
}

impl VecPos {
    #[inline(always)]
    fn new(v: usize, beg: usize, len: usize) -> Self {
        Self { v, beg, len }
    }
}
