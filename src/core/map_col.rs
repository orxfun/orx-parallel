use super::{
    diagnostics::ParThreadLogger,
    runner::{ParTask, Runner},
};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;

pub fn map_col<I, Out, Map, P>(
    params: Params,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    match params.is_sequential() {
        true => seq_map_col(iter, map, collected),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_map_col::<_, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, collected,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_map_col::<_, _, _, _, super::diagnostics::NoLogger>(params, iter, map, collected)
        }
    }
}

fn par_map_col<I, Out, Map, P, L>(
    params: Params,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
    L: ParThreadLogger,
{
    let offset = collected.len();
    let task = |c| task::<_, _, _, _, L>(&iter, &map, &collected, offset, c);
    let num_spawned = Runner::run(params, ParTask::Collect, &iter, &task);

    L::log_num_spawned(num_spawned);
    unsafe { collected.into_inner().unwrap_only_if_counts_match() }
}

fn task<I, Out, Map, P, L>(
    iter: &I,
    map: &Map,
    collected: &ConcurrentOrderedBag<Out, P>,
    offset: usize,
    chunk_size: usize,
) where
    I: ConcurrentIter,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            iter.ids_and_values()
                .map(|(idx, value)| (offset + idx, map(value)))
                .for_each(|(idx, value)| unsafe { collected.set_value(idx, value) });
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                let begin_idx = offset + chunk.begin_idx;
                unsafe { collected.set_values(begin_idx, chunk.values.map(&map)) };
            }
        }
    }
}

fn seq_map_col<I, Out, Map, P>(iter: I, map: Map, collected: ConcurrentOrderedBag<Out, P>) -> P
where
    I: ConcurrentIter,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    let mut output = unsafe { collected.into_inner().unwrap_only_if_counts_match() };
    let iter = iter.into_seq_iter();
    for x in iter.map(map) {
        output.push(x);
    }
    output
}
