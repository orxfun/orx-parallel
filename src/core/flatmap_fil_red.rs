use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn fmap_fil_red<I, OutIter, Out, Map, Fil, Red>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_fmap_fil_red(iter, fmap, filter, reduce),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_fmap_fil_red::<_, _, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, fmap, filter, reduce,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_fmap_fil_red::<_, _, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, fmap, filter, reduce,
            )
        }
    }
}

fn par_fmap_fil_red<I, OutIter, Out, Map, Fil, Red, L>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, _, L>(&iter, &fmap, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    L::log_num_spawned(num_spawned);
    reduced.flatten()
}

fn task<I, OutIter, Out, Map, Fil, Red, L>(
    iter: &I,
    fmap: &Map,
    filter: &Fil,
    reduce: &Red,
    chunk_size: usize,
) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => iter.values().flat_map(fmap).filter(filter).reduce(reduce),
        c => {
            let mut acc = None;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                let x = chunk.values.flat_map(fmap).filter(filter).reduce(reduce);
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
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter,
    Fil: Fn(&Out) -> bool,
    Red: Fn(Out, Out) -> Out,
{
    let iter = iter.into_seq_iter();
    iter.flat_map(fmap).filter(filter).reduce(reduce)
}
