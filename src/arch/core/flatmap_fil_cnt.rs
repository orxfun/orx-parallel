use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIterX;

pub fn fmap_fil_cnt<I, OutIter, Out, Map, Fil>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> usize
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_fmap_fil_cnt(iter, fmap, filter),
        false => par_fmap_fil_cnt(params, iter, fmap, filter),
    }
}

fn par_fmap_fil_cnt<I, OutIter, Out, Map, Fil>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> usize
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &fmap, &filter, c);
    let reduce = |a, b| a + b;
    let (_num_spawned, count) = Runner::reduce(params, ParTask::Collect, &iter, &task, reduce);

    count.unwrap_or(0)
}

fn task<I, OutIter, Out, Map, Fil>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> usize
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => iter.values().flat_map(&map).filter(&filter).count(),
        c => {
            let mut count = 0;
            while let Some(chunk) = iter.next_chunk_x(c) {
                count += chunk.flat_map(&map).filter(&filter).count();
            }
            count
        }
    }
}

fn seq_fmap_fil_cnt<I, OutIter, Out, Map, Fil>(iter: I, fmap: Map, filter: Fil) -> usize
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    iter.flat_map(fmap).filter(filter).count()
}
