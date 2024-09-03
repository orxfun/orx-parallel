use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterX};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub fn map_col<I, Out, Map, P>(
    params: Params,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: IntoConcurrentPinnedVec<Out>,
{
    match params.is_sequential() {
        true => seq_map_col(iter, map, collected),
        false => {
            let offset = collected.len();
            let task = |c| task(&iter, &map, &collected, offset, c);
            let _num_spawned = Runner::run(params, ParTask::Collect, &iter, &task);
            unsafe { collected.into_inner().unwrap_only_if_counts_match() }
        }
    }
}

fn task<I, Out, Map, P>(
    iter: &I,
    map: &Map,
    collected: &ConcurrentOrderedBag<Out, P>,
    offset: usize,
    chunk_size: usize,
) where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: IntoConcurrentPinnedVec<Out>,
{
    match chunk_size {
        1 => {
            iter.ids_and_values()
                .map(|(idx, value)| (offset + idx, map(value)))
                .for_each(|(idx, value)| unsafe { collected.set_value(idx, value) });
        }
        c => {
            while let Some(chunk) = iter.next_chunk(c) {
                let begin_idx = offset + chunk.begin_idx;
                unsafe { collected.set_values(begin_idx, chunk.values.map(&map)) };
            }
        }
    }
}

fn seq_map_col<I, Out, Map, P>(iter: I, map: Map, collected: ConcurrentOrderedBag<Out, P>) -> P
where
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: IntoConcurrentPinnedVec<Out>,
{
    let mut output = unsafe { collected.into_inner().unwrap_only_if_counts_match() };
    let iter = iter.into_seq_iter();
    for x in iter.map(map) {
        output.push(x);
    }
    output
}
