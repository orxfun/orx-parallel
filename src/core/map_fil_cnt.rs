use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_cnt<I, Out, Map, Fil>(params: Params, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_cnt(iter, map, filter),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_map_fil_cnt::<_, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, filter,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_map_fil_cnt::<_, _, _, _, super::diagnostics::NoLogger>(params, iter, map, filter)
        }
    }
}

fn par_map_fil_cnt<I, Out, Map, Fil, L>(params: Params, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, L>(&iter, &map, &filter, c);
    let reduce = |a, b| a + b;
    let (num_spawned, count) = Runner::reduce(params, ParTask::Collect, &iter, &task, reduce);

    L::log_num_spawned(num_spawned);
    count.unwrap_or(0)
}

fn task<I, Out, Map, Fil, L>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => iter.values().map(&map).filter(&filter).count(),
        c => {
            let mut count = 0;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                count += chunk.values.map(&map).filter(&filter).count();
            }
            count
        }
    }
}

fn seq_map_fil_cnt<I, Out, Map, Fil>(iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter().map(map).filter(filter).count()
}
