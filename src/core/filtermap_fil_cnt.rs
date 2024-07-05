use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::{Fallible, Params};
use orx_concurrent_iter::ConcurrentIter;

pub fn filtermap_fil_cnt<I, FO, Out, FilterMap, Fil>(
    params: Params,
    iter: I,
    map: FilterMap,
    filter: Fil,
) -> usize
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_filtermap_fil_cnt(iter, map, filter),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_filtermap_fil_cnt::<_, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, filter,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_filtermap_fil_cnt::<_, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, map, filter,
            )
        }
    }
}

fn par_filtermap_fil_cnt<I, FO, Out, FilterMap, Fil, L>(
    params: Params,
    iter: I,
    map: FilterMap,
    filter: Fil,
) -> usize
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, L>(&iter, &map, &filter, c);
    let reduce = |a, b| a + b;
    let (num_spawned, count) = Runner::reduce(params, ParTask::Reduce, &iter, &task, reduce);

    L::log_num_spawned(num_spawned);
    count.unwrap_or(0)
}

fn task<I, FO, Out, FilterMap, Fil, L>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    chunk_size: usize,
) -> usize
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync + Clone,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            for x in iter.values() {
                let maybe = filter_map(x);
                if maybe.has_value() {
                    let x = maybe.value();
                    if filter(&x) {
                        let mut acc = 1;

                        for x in iter.values() {
                            let maybe = filter_map(x);
                            if maybe.has_value() {
                                let x = maybe.value();
                                if filter(&x) {
                                    acc += 1;
                                }
                            }
                        }

                        return acc;
                    }
                }
            }
            0
        }
        c => {
            let mut acc = 0;
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                let x = chunk
                    .values
                    .map(filter_map)
                    .filter(|x| x.has_value())
                    .map(|x| x.value())
                    .filter(filter)
                    .count();
                acc += x;
            }
            acc
        }
    }
}

fn seq_filtermap_fil_cnt<I, FO, Out, FilterMap, Fil>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
) -> usize
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter()
        .map(filter_map)
        .filter(|x| x.has_value())
        .map(|x| x.value())
        .filter(filter)
        .count()
}
