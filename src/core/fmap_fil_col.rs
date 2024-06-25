use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;
use std::cmp::Ordering;

pub fn par_fmap_fil_col<I, OutIter, Out, Map, Fil, P, Q>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
    pos_len: ConcurrentOrderedBag<(usize, usize), Q>,
) -> (P, Q)
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<(usize, usize)>,
{
    #[cfg(feature = "with_diagnostics")]
    return par_fmap_fil_col_core::<_, _, _, _, _, _, _, super::diagnostics::ParLogger>(
        params, iter, map, filter, collected, pos_len,
    );

    #[cfg(not(feature = "with_diagnostics"))]
    par_fmap_fil_col_core::<_, _, _, _, _, _, _, super::diagnostics::NoLogger>(
        params, iter, map, filter, collected, pos_len,
    )
}

fn par_fmap_fil_col_core<I, OutIter, Out, Map, Fil, P, Q, L>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
    pos_len: ConcurrentOrderedBag<(usize, usize), Q>,
) -> (P, Q)
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<(usize, usize)>,
    L: ParThreadLogger,
{
    let offset = collected.len();
    let task =
        |c| task::<_, _, _, _, _, _, _, L>(&iter, &map, &filter, &collected, &pos_len, offset, c);
    let num_spawned = Runner::run(params, ParTask::Collect, &iter, &task);

    L::log_num_spawned(num_spawned);
    (collected.into_inner(), unsafe {
        pos_len.into_inner().unwrap_only_if_counts_match()
    })
}

fn task<I, OutIter, Out, Map, Fil, P, Q, L>(
    iter: &I,
    map: &Map,
    filter: &Fil,
    collected: &ConcurrentBag<Out, P>,
    pos_len: &ConcurrentOrderedBag<(usize, usize), Q>,
    offset: usize,
    chunk_size: usize,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<(usize, usize)>,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                let idx = offset + x.idx;
                let buffer: Vec<_> = map(x.value).into_iter().filter(filter).collect();
                let len = buffer.len();
                let begin_position = collected.extend(buffer);
                unsafe { pos_len.set_value(idx, (begin_position, len)) };
            }
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            let mut local = vec![(usize::MAX, 0); c];
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());

                let count = chunk.values.len();
                let begin_idx = offset + chunk.begin_idx;
                let buffer: Vec<Out> = chunk.values.flat_map(map).filter(filter).collect();
                let buffer_len = buffer.len();
                let begin_position = collected.extend(buffer);
                local[0] = (begin_position, buffer_len);
                unsafe {
                    match count.cmp(&c) {
                        Ordering::Less => {
                            pos_len.set_values(begin_idx, local.iter().take(count).cloned())
                        }
                        _ => pos_len.set_values(begin_idx, local.iter().cloned()),
                    }
                };
            }
        }
    }
}

pub fn seq_fmap_fil_col<I, OutIter, Out, Map, Fil, Output, Push>(
    iter: I,
    map: Map,
    filter: Fil,
    output: &mut Output,
    mut push: Push,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Push: FnMut(&mut Output, Out),
{
    let iter = iter.into_seq_iter();
    for x in iter.flat_map(map).filter(filter) {
        push(output, x);
    }
}
