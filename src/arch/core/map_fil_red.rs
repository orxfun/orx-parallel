use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIterX;

pub fn map_fil_red<I, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_red(iter, map, filter, reduce),
        false => par_map_fil_red(params, iter, map, filter, reduce),
    }
}

fn par_map_fil_red<I, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    let task = |c| task(&iter, &map, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (_num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    reduced.flatten()
}

fn task<I, Out, Map, Fil, Red>(
    iter: &I,
    map: &Map,
    filter: &Fil,
    reduce: &Red,
    chunk_size: usize,
) -> Option<Out>
where
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match chunk_size {
        1 => iter.values().map(map).filter(filter).reduce(reduce),
        c => {
            let mut acc = None;
            while let Some(chunk) = iter.next_chunk_x(c) {
                let x = chunk.map(map).filter(filter).reduce(reduce);
                acc = maybe_reduce(reduce, acc, x);
            }
            acc
        }
    }
}

fn seq_map_fil_red<I, Out, Map, Fil, Red>(
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    iter.into_seq_iter().map(map).filter(filter).reduce(reduce)
}
