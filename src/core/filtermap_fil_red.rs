use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
use crate::fn_sync::FnSync;
use crate::{Fallible, Params};
use orx_concurrent_iter::ConcurrentIter;

pub fn filtermap_fil_red<I, FO, Out, FilterMap, Fil, Red>(
    params: Params,
    iter: I,
    map: FilterMap,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + FnSync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_filtermap_fil_red(iter, map, filter, reduce),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_filtermap_fil_red::<_, _, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, filter, reduce,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_filtermap_fil_red::<_, _, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, map, filter, reduce,
            )
        }
    }
}

fn par_filtermap_fil_red<I, FO, Out, FilterMap, Fil, Red, L>(
    params: Params,
    iter: I,
    map: FilterMap,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + FnSync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, _, L>(&iter, &map, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    L::log_num_spawned(num_spawned);
    reduced.flatten()
}

fn task<I, FO, Out, FilterMap, Fil, Red, L>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    reduce: &Red,
    chunk_size: usize,
) -> Option<Out>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + FnSync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            for x in iter.values() {
                let maybe = filter_map(x);
                if maybe.has_value() {
                    let x = maybe.unwrap();
                    if filter(&x) {
                        let mut acc = x;

                        for x in iter.values() {
                            let maybe = filter_map(x);
                            if maybe.has_value() {
                                let x = maybe.unwrap();
                                if filter(&x) {
                                    acc = reduce(acc, x);
                                }
                            }
                        }

                        return Some(acc);
                    }
                }
            }
            None
        }
        c => {
            let mut acc = None;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                let x = chunk
                    .values
                    .map(filter_map)
                    .filter(|x| x.has_value())
                    .map(|x| x.unwrap())
                    .filter(filter)
                    .reduce(reduce);
                acc = maybe_reduce(reduce, acc, x);
            }
            acc
        }
    }
}

fn seq_filtermap_fil_red<I, FO, Out, FilterMap, Fil, Red>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    iter.into_seq_iter()
        .map(filter_map)
        .filter(|x| x.has_value())
        .map(|x| x.unwrap())
        .filter(filter)
        .reduce(reduce)
}
