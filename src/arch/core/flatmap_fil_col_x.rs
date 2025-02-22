use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIterX;
use orx_split_vec::{Recursive, SplitVec};

fn task<I, OutIter, Out, FlatMap, Fil>(
    iter: &I,
    flat_map: &FlatMap,
    filter: &Fil,
    chunk_size: usize,
) -> Vec<Out>
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => iter.values().flat_map(&flat_map).filter(&filter).collect(),
        c => {
            let mut collected = vec![];
            while let Some(chunk) = iter.next_chunk_x(c) {
                collected.extend(chunk.flat_map(&flat_map).filter(&filter));
            }
            collected
        }
    }
}

pub fn par_flatmap_fil_col_x_rec<I, OutIter, Out, FlatMap, Fil>(
    params: Params,
    iter: I,
    flat_map: FlatMap,
    filter: Fil,
    output: &mut SplitVec<Out, Recursive>,
) where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &flat_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    output.append(vectors);
}
