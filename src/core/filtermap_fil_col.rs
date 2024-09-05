use super::map_fil_col::{heap_sort_into_pinned_vec, heap_sort_into_vec};
use super::runner::{ParTask, Runner};
use crate::{Fallible, Params};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterX};
use orx_fixed_vec::PinnedVec;

fn task<I, FO, Out, FilterMap, Fil>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    chunk_size: usize,
) -> Vec<(usize, Out)>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let mut collected = vec![];
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                let maybe = filter_map(x.value);
                if maybe.has_value() {
                    let value = maybe.value();
                    if filter(&value) {
                        collected.push((x.idx, value));
                    }
                }
            }
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                for (c, value) in chunk.values.enumerate() {
                    let maybe = filter_map(value);
                    if maybe.has_value() {
                        let value = maybe.value();
                        if filter(&value) {
                            collected.push((chunk.begin_idx + c, value));
                        }
                    }
                }
            }
        }
    }
    collected
}

pub fn par_filtermap_fil_col_vec<I, FO, Out, FilterMap, Fil>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut Vec<Out>,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &filter_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    heap_sort_into_vec(vectors, output);
}

pub fn par_filtermap_fil_col_pinned_vec<I, FO, Out, FilterMap, Fil, P>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut P,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let task = |c| task(&iter, &filter_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    heap_sort_into_pinned_vec(vectors, output);
}

pub fn seq_filtermap_fil_col_vec<I, FO, Out, FilterMap, Fil>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut Vec<Out>,
) where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter
        .into_seq_iter()
        .map(filter_map)
        .filter(|x| x.has_value())
        .map(|x| x.value())
        .filter(filter);
    for x in iter {
        output.push(x);
    }
}

pub fn seq_filtermap_fil_col_pinned_vec<I, FO, Out, FilterMap, Fil, P>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    output: &mut P,
) where
    I: ConcurrentIterX,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let iter = iter
        .into_seq_iter()
        .map(filter_map)
        .filter(|x| x.has_value())
        .map(|x| x.value())
        .filter(filter);
    for x in iter {
        output.push(x);
    }
}
