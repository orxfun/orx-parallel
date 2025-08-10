use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};

pub fn heap_sort_into<T, P>(
    mut vectors: Vec<Vec<(usize, T)>>,
    max_idx_exc: Option<usize>,
    output: &mut P,
) where
    P: PinnedVec<T>,
{
    let mut queue = BinaryHeap::with_capacity(vectors.len());
    let mut indices = vec![0; vectors.len()];

    let max_idx_or_inf = max_idx_exc.unwrap_or(usize::MAX);
    for (v, vec) in vectors.iter().enumerate() {
        if let Some(x) = vec.get(indices[v])
            && x.0 < max_idx_or_inf
        {
            queue.push(v, x.0);
        }
    }
    let mut curr_v = queue.pop_node();

    match max_idx_exc {
        None => {
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
        }
        Some(max_idx) => {
            while let Some(v) = curr_v {
                let idx = indices[v];
                indices[v] += 1;

                curr_v = match vectors[v].get(indices[v]) {
                    Some(x) if x.0 < max_idx => Some(queue.push_then_pop(v, x.0).0),
                    _ => queue.pop_node(),
                };

                let ptr = vectors[v].as_ptr();
                output.push(unsafe { ptr.add(idx).read().1 });
            }
        }
    }

    for vec in vectors.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.set_len(0) };
    }
}
