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
        false => par_map_fil_cnt(params, iter, map, filter),
    }
}

fn par_map_fil_cnt<I, Out, Map, Fil>(params: Params, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &map, &filter, c);
    let reduce = |a, b| a + b;
    let (_num_spawned, count) = Runner::reduce(params, ParTask::Collect, &iter, &task, reduce);

    count.unwrap_or(0)
}

fn task<I, Out, Map, Fil>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => iter.values().map(&map).filter(&filter).count(),
        c => {
            let mut count = 0;
            while let Some(chunk) = iter.next_chunk(c) {
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
