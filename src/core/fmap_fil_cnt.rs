use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn fmap_fil_cnt<I, OutIter, Out, Map, Fil>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> usize
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_fmap_fil_cnt(iter, fmap, filter),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_fmap_fil_cnt::<_, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, fmap, filter,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_fmap_fil_cnt::<_, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, fmap, filter,
            )
        }
    }
}

fn par_fmap_fil_cnt<I, OutIter, Out, Map, Fil, L>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> usize
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, L>(&iter, &fmap, &filter, c);
    let reduce = |a, b| a + b;
    let (num_spawned, count) = Runner::reduce(params, ParTask::Collect, &iter, &task, reduce);

    L::log_num_spawned(num_spawned);
    count.unwrap_or(0)
}

fn task<I, OutIter, Out, Map, Fil, L>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> usize
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => iter.values().flat_map(&map).filter(&filter).count(),
        c => {
            let mut count = 0;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                count += chunk.values.flat_map(&map).filter(&filter).count();
            }
            count
        }
    }
}

fn seq_fmap_fil_cnt<I, OutIter, Out, Map, Fil>(iter: I, fmap: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    iter.flat_map(fmap).filter(filter).count()
}
