use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};

pub fn heap_sort_into<T, P>(mut vectors: Vec<Vec<(usize, T)>>, output: &mut P)
where
    P: PinnedVec<T>,
{
    let mut queue = BinaryHeap::with_capacity(vectors.len());
    let mut indices = vec![0; vectors.len()];

    for (v, vec) in vectors.iter().enumerate() {
        if let Some(x) = vec.get(indices[v]) {
            queue.push(v, x.0);
        }
    }
    let mut curr_v = queue.pop_node();

    while let Some(v) = curr_v {
        let idx = indices[v];
        indices[v] += 1;

        curr_v = match vectors[v].get(indices[v]) {
            Some(x) => Some(queue.push_then_pop(v, x.0).0),
            None => queue.pop_node(),
        };

        let ptr = vectors[v].as_ptr();
        output.push(unsafe { ptr.add(idx).read().1 });
    }

    for vec in vectors.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.set_len(0) };
    }
}
