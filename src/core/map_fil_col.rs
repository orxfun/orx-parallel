use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use std::fmt::Debug;

fn task<I, Out, Map, Fil>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> Vec<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let mut collected = vec![];
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                let value = map(x.value);
                if filter(&value) {
                    collected.push((x.idx, value));
                };
            }
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                for (i, value) in chunk.values.map(map).filter(filter).enumerate() {
                    collected.push((chunk.begin_idx + i, value));
                }
            }
        }
    }
    collected
}

fn join_vec<Out>(mut vectors: Vec<Vec<(usize, Out)>>, output: &mut Vec<Out>) {
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

        let ptr = vectors[v].as_mut_ptr();
        output.push(unsafe { ptr.add(idx).read().1 });
    }

    for vec in vectors.iter_mut() {
        unsafe { vec.set_len(0) };
    }
}

fn join_pinned_vec<Out, P>(mut vectors: Vec<Vec<(usize, Out)>>, output: &mut P)
where
    P: PinnedVec<Out>,
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

        let ptr = vectors[v].as_mut_ptr();
        output.push(unsafe { ptr.add(idx).read().1 });
    }

    for vec in vectors.iter_mut() {
        unsafe { vec.set_len(0) };
    }
}

pub fn par_map_fil_col_vec<I, Out, Map, Fil>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    output: &mut Vec<Out>,
) where
    I: ConcurrentIter,
    Out: Send + Sync + Debug,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    join_vec(vectors, output);
}

pub fn par_map_fil_col_pinned_vec<I, Out, Map, Fil, P>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    output: &mut P,
) where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let task = |c| task(&iter, &map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    join_pinned_vec(vectors, output);
}

pub fn seq_map_fil_col_vec<I, Out, Map, Fil>(iter: I, map: Map, filter: Fil, output: &mut Vec<Out>)
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    for x in iter.map(map).filter(filter) {
        output.push(x);
    }
}

pub fn seq_map_fil_col_pinned_vec<I, Out, Map, Fil, Output>(
    iter: I,
    map: Map,
    filter: Fil,
    output: &mut Output,
) where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Output: PinnedVec<Out>,
{
    let iter = iter.into_seq_iter();
    for x in iter.map(map).filter(filter) {
        output.push(x);
    }
}
