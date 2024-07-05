use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

fn task<I, Out, Map, Fil>(iter: &I, map: &Map, filter: &Fil, chunk_size: usize) -> Vec<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => iter.values().map(&map).filter(&filter).collect(),
        c => {
            let mut collected = vec![];
            while let Some(chunk) = iter.next_chunk(c) {
                collected.extend(chunk.values.map(&map).filter(&filter));
            }
            collected
        }
    }
}

pub fn par_map_fil_col_x_rec<I, Out, Map, Fil>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    output: &mut SplitVec<Out, Recursive>,
) where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    output.append(vectors);
}
