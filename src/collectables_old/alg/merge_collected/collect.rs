use crate::collectables_old::{Collectable, vals_and_idx::ValsAndIdx};
use alloc::{vec, vec::Vec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

pub struct Collect;

impl Collect {
    pub fn merge_results_arb<T, S, D>(results: Vec<S>, dst: &mut D)
    where
        S: Collectable<T>,
        D: Collectable<T>,
    {
        let total_len: usize = results.iter().map(|x| x.col_len()).sum();
        dst.col_reserve(total_len);
        for vec in results {
            dst.col_extend(vec);
        }
    }

    pub fn merge_results<T, S, D>(mut results: Vec<ValsAndIdx<T, S>>, dst: &mut D)
    where
        T: Send,
        S: Collectable<T>,
        D: Collectable<T>,
    {
        todo!()
        // let collected_len: usize = results.iter().map(|x| x.values.col_len()).sum();
        // dst.col_reserve(collected_len);
        // let initial_len = dst.col_len();
        // let total_len = initial_len + collected_len;

        // let mut queue = BinaryHeap::with_capacity(results.len());
        // let mut pos_indices = vec![0; results.len()];

        // for (v, vec) in results.iter().enumerate() {
        //     if let Some(pos) = vec.positions.first() {
        //         queue.push(VecPos::new(v, 0, pos.len), pos.idx);
        //     }
        // }
        // let mut curr_v = queue.pop_node();
        // let mut ptr_dst = unsafe { dst.as_mut_ptr().add(initial_len) };

        // while let Some(VecPos { v, beg, len }) = curr_v {
        //     let ptr_src = unsafe { results[v].values.as_ptr().add(beg) };
        //     unsafe { ptr_dst.copy_from_nonoverlapping(ptr_src, len) };

        //     pos_indices[v] += 1;
        //     curr_v = match results[v].positions.get(pos_indices[v]) {
        //         Some(pos) => {
        //             let beg = beg + len;
        //             Some(queue.push_then_pop(VecPos::new(v, beg, pos.len), pos.idx).0)
        //         }
        //         None => queue.pop_node(),
        //     };

        //     ptr_dst = unsafe { ptr_dst.add(len) };
        // }

        // for vec in results.iter_mut() {
        //     // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        //     // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        //     unsafe { vec.values.set_len(0) };
        // }

        // unsafe { dst.set_len(total_len) };
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
