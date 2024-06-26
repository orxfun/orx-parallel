use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_red<I, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_red(iter, map, filter, reduce),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_map_fil_red::<_, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, filter, reduce,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_map_fil_red::<_, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, map, filter, reduce,
            )
        }
    }
}

fn par_map_fil_red<I, Out, Map, Fil, Red, L>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, L>(&iter, &map, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    L::log_num_spawned(num_spawned);
    reduced.flatten()
}

fn task<I, Out, Map, Fil, Red, L>(
    iter: &I,
    map: &Map,
    filter: &Fil,
    reduce: &Red,
    chunk_size: usize,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => iter.values().map(map).filter(filter).reduce(reduce),
        c => {
            let mut acc = None;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                let x = chunk.values.map(map).filter(filter).reduce(reduce);
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
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    iter.into_seq_iter().map(map).filter(filter).reduce(reduce)
}
