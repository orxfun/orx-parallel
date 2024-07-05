use super::map_fil_col::{heap_sort_into_pinned_vec, heap_sort_into_vec};
use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::PinnedVec;

fn task<I, OutIter, Out, FlatMap, Fil>(
    iter: &I,
    flat_map: &FlatMap,
    filter: &Fil,
    chunk_size: usize,
) -> Vec<((usize, usize), Out)>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let mut collected = vec![];
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                collected.extend(
                    flat_map(x.value)
                        .into_iter()
                        .filter(filter)
                        .enumerate()
                        .map(|(i, value)| ((x.idx, i), value)),
                );
            }
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                for (c, value) in chunk.values.enumerate() {
                    collected.extend(
                        flat_map(value)
                            .into_iter()
                            .filter(filter)
                            .enumerate()
                            .map(|(i, value)| ((chunk.begin_idx + c, i), value)),
                    );
                }
            }
        }
    }
    collected
}

pub fn par_flatmap_fil_col_vec<I, OutIter, Out, FlatMap, Fil>(
    params: Params,
    iter: I,
    flat_map: FlatMap,
    filter: Fil,
    output: &mut Vec<Out>,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &flat_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    heap_sort_into_vec(vectors, output);
}

pub fn par_flatmap_fil_col_pinned_vec<I, OutIter, Out, FlatMap, Fil, P>(
    params: Params,
    iter: I,
    flat_map: FlatMap,
    filter: Fil,
    output: &mut P,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let task = |c| task(&iter, &flat_map, &filter, c);
    let vectors = Runner::run_map(params, ParTask::Collect, &iter, &task);
    heap_sort_into_pinned_vec(vectors, output);
}

pub fn seq_flatmap_fil_col_vec<I, OutIter, Out, FlatMap, Fil>(
    iter: I,
    flat_map: FlatMap,
    filter: Fil,
    output: &mut Vec<Out>,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    for x in iter.flat_map(flat_map).filter(filter) {
        output.push(x);
    }
}

pub fn seq_flatmap_fil_col_pinned_vec<I, OutIter, Out, FlatMap, Fil, P>(
    iter: I,
    flat_map: FlatMap,
    filter: Fil,
    output: &mut P,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    FlatMap: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let iter = iter.into_seq_iter();
    for x in iter.flat_map(flat_map).filter(filter) {
        output.push(x);
    }
}
