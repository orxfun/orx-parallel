use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIterX;

pub fn fmap_fil_red<I, OutIter, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_fmap_fil_red(iter, fmap, filter, reduce),
        false => par_fmap_fil_red(params, iter, fmap, filter, reduce),
    }
}

fn par_fmap_fil_red<I, OutIter, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    let task = |c| task(&iter, &fmap, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (_num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    reduced.flatten()
}

fn task<I, OutIter, Out, Map, Fil, Red>(
    iter: &I,
    fmap: &Map,
    filter: &Fil,
    reduce: &Red,
    chunk_size: usize,
) -> Option<Out>
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match chunk_size {
        1 => iter.values().flat_map(fmap).filter(filter).reduce(reduce),
        c => {
            let mut acc = None;
            let mut buffered = iter.buffered_iter_x(c);
            while let Some(chunk) = buffered.next_x() {
                let x = chunk.flat_map(fmap).filter(filter).reduce(reduce);
                acc = maybe_reduce(reduce, acc, x);
            }
            acc
        }
    }
}

fn seq_fmap_fil_red<I, OutIter, Out, Map, Fil, Red>(
    iter: I,
    fmap: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter,
    Fil: Fn(&Out) -> bool,
    Red: Fn(Out, Out) -> Out,
{
    let iter = iter.into_seq_iter();
    iter.flat_map(fmap).filter(filter).reduce(reduce)
}
