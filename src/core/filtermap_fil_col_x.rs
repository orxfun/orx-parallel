use super::runner::{ParTask, Runner};
use crate::{Fallible, Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{Recursive, SplitVec};

fn task<I, FO, Out, FilterMap, Fil>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    chunk_size: usize,
) -> Vec<Out>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => iter
            .values()
            .map(&filter_map)
            .filter(|x| x.has_value())
            .map(|x| x.value())
            .filter(&filter)
            .collect(),
        c => {
            let mut collected = vec![];
            while let Some(chunk) = iter.next_chunk(c) {
                collected.extend(
                    chunk
                        .values
                        .map(&filter_map)
                        .filter(|x| x.has_value())
                        .map(|x| x.value())
                        .filter(&filter),
                );
            }
            collected
        }
    }
}

pub fn par_filtermap_fil_col_x_rec<I, FO, Out, FilterMap, Fil>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut SplitVec<Out, Recursive>,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &filter_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    output.append(vectors);
}
