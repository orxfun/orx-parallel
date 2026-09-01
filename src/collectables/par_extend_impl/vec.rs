use crate::collectables::par_extend::ParExtend;
use crate::collectables::par_extend_impl::{col_and_pos::ColAndPos, idx_len::IdxLen};
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

impl<T> ParExtend<T> for Vec<T> {
    type ThreadValues = Self;

    type OrderedThreadValues = ColAndPos<Vec<T>>;

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

    // extend

    fn extend_from_ordered_thread_results(&mut self, mut results: Vec<Self::OrderedThreadValues>) {
        let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
        self.reserve(collected_len);
        let initial_len = self.len();
        let total_len = initial_len + collected_len;

        let mut queue = BinaryHeap::with_capacity(results.len());
        let mut pos_indices = vec![0; results.len()];

        for (t, vec) in results.iter().enumerate() {
            if let Some(pos) = vec.positions.first() {
                queue.push(VecPos::new(t, 0, pos.len), pos.idx);
            }
        }
        let mut curr_t = queue.pop_node();
        let mut ptr_dst = unsafe { self.as_mut_ptr().add(initial_len) };

        while let Some(VecPos { t, beg, len }) = curr_t {
            let ptr_src = unsafe { results[t].values.as_ptr().add(beg) };
            unsafe { ptr_dst.copy_from_nonoverlapping(ptr_src, len) };

            pos_indices[t] += 1;
            curr_t = match results[t].positions.get(pos_indices[t]) {
                Some(pos) => {
                    let beg = beg + len;
                    Some(queue.push_then_pop(VecPos::new(t, beg, pos.len), pos.idx).0)
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

// merge helpers

#[derive(Clone)]
struct VecPos {
    t: usize,
    beg: usize,
    len: usize,
}

impl VecPos {
    #[inline(always)]
    fn new(t: usize, beg: usize, len: usize) -> Self {
        Self { t, beg, len }
    }
}
