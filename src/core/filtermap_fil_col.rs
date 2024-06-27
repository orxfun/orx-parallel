use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::{Fallible, Params};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;
use std::cmp::Ordering;
use std::fmt::Debug;

pub fn par_filtermap_fil_col<I, FO, Out, FilterMap, Fil, P, Q>(
    params: Params,
    iter: I,
    map: FilterMap,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
    positions: ConcurrentOrderedBag<usize, Q>,
) -> (P, Q)
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync + Debug,
    Out: Send + Sync + Debug,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<usize>,
{
    #[cfg(feature = "with_diagnostics")]
    return par_filtermap_fil_col_core::<_, _, _, _, _, _, _, super::diagnostics::ParLogger>(
        params, iter, map, filter, collected, positions,
    );

    #[cfg(not(feature = "with_diagnostics"))]
    par_filtermap_fil_col_core::<_, _, _, _, _, _, _, super::diagnostics::NoLogger>(
        params, iter, map, filter, collected, positions,
    )
}

fn par_filtermap_fil_col_core<I, FO, Out, FilterMap, Fil, P, Q, L>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
    positions: ConcurrentOrderedBag<usize, Q>,
) -> (P, Q)
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync + Debug,
    Out: Send + Sync + Debug,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<usize>,
    L: ParThreadLogger,
{
    let offset = collected.len();
    let task = |c| {
        task::<_, _, _, _, _, _, _, L>(
            &iter,
            &filter_map,
            &filter,
            &collected,
            &positions,
            offset,
            c,
        )
    };
    let num_spawned = Runner::run(params, ParTask::Collect, &iter, &task);

    L::log_num_spawned(num_spawned);
    (collected.into_inner(), unsafe {
        positions.into_inner().unwrap_only_if_counts_match()
    })
}

fn task<I, FO, Out, FilterMap, Fil, P, Q, L>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    collected: &ConcurrentBag<Out, P>,
    positions: &ConcurrentOrderedBag<usize, Q>,
    offset: usize,
    chunk_size: usize,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync + Debug,
    Out: Send + Sync + Debug,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<usize>,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                let maybe = filter_map(x.value);
                let position = match maybe.has_value() {
                    false => usize::MAX,
                    true => {
                        let value = maybe.unwrap();
                        match filter(&value) {
                            true => collected.push(value),
                            false => usize::MAX,
                        }
                    }
                };
                let idx = offset + x.idx;
                unsafe { positions.set_value(idx, position) };
            }
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            let mut local = vec![0usize; c];
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());

                let count = chunk.values.len();
                let begin_idx = offset + chunk.begin_idx;

                for (i, value) in chunk.values.enumerate() {
                    let maybe = filter_map(value);
                    let position = match maybe.has_value() {
                        false => usize::MAX,
                        true => {
                            let value = maybe.unwrap();
                            match filter(&value) {
                                true => collected.push(value),
                                false => usize::MAX,
                            }
                        }
                    };
                    local[i] = position;
                }

                unsafe {
                    match count.cmp(&c) {
                        Ordering::Less => {
                            // TODO: memcpy here! all .set_values calls can improve by memcpy
                            positions.set_values(begin_idx, local.iter().take(count).copied())
                        }
                        _ => positions.set_values(begin_idx, local.iter().copied()),
                    }
                };
            }
        }
    }
}

pub fn seq_filtermap_fil_col<I, FO, Out, FilterMap, Fil, Output, Push>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut Output,
    mut push: Push,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync + Debug,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Push: FnMut(&mut Output, Out),
{
    let iter = iter.into_seq_iter();
    for x in iter
        .map(filter_map)
        .filter(|x| x.has_value())
        .map(|x| x.unwrap())
        .filter(filter)
    {
        push(output, x);
    }
}
