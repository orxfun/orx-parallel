use super::runner::{ParTask, Runner};
use super::utils::maybe_reduce;
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
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_filtermap_fil_red(iter, map, filter, reduce),
        false => par_filtermap_fil_red(params, iter, map, filter, reduce),
    }
}

fn par_filtermap_fil_red<I, FO, Out, FilterMap, Fil, Red>(
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
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    let task = |c| task(&iter, &map, &filter, &reduce, c);
    let reduce_outer = |a: Option<Out>, b: Option<Out>| maybe_reduce(&reduce, a, b);
    let (_num_spawned, reduced) =
        Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce_outer);

    reduced.flatten()
}

fn task<I, FO, Out, FilterMap, Fil, Red>(
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
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match chunk_size {
        1 => {
            for x in iter.values() {
                let maybe = filter_map(x);
                if maybe.has_value() {
                    let x = maybe.value();
                    if filter(&x) {
                        let mut acc = x;

                        for x in iter.values() {
                            let maybe = filter_map(x);
                            if maybe.has_value() {
                                let x = maybe.value();
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
                let x = chunk
                    .values
                    .map(filter_map)
                    .filter(|x| x.has_value())
                    .map(|x| x.value())
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
        .map(|x| x.value())
        .filter(filter)
        .reduce(reduce)
}
