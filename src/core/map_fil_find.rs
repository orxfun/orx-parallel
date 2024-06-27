use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::core::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_find<I, Out, Map, Fil>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_find(iter, map, filter),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_map_fil_find::<_, _, _, _, super::diagnostics::ParLogger>(
                params, iter, map, filter,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_map_fil_find::<_, _, _, _, super::diagnostics::NoLogger>(params, iter, map, filter)
        }
    }
}

fn par_map_fil_find<I, Out, Map, Fil, L>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, L>(&iter, &map, &filter, c);
    let reduce =
        |a: Option<(usize, _)>, b| maybe_reduce(|a, b| if b.0 < a.0 { b } else { a }, a, b);
    let (num_spawned, found) = Runner::reduce(params, ParTask::EarlyReturn, &iter, &task, reduce);

    L::log_num_spawned(num_spawned);
    found.flatten()
}

fn task<I, Out, Map, Fil, L>(
    iter: &I,
    map: &Map,
    filter: &Fil,
    chunk_size: usize,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            let result = iter
                .ids_and_values()
                .map(|x| (x.0, map(x.1)))
                .find(|x| filter(&x.1));

            if result.is_some() {
                iter.skip_to_end();
            }

            result
        }
        c => {
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());

                let result = chunk
                    .values
                    .enumerate()
                    .map(|x| (x.0, map(x.1)))
                    .find(|x| filter(&x.1))
                    .map(|x| (chunk.begin_idx + x.0, x.1));

                if result.is_some() {
                    iter.skip_to_end();
                    return result;
                }
            }
            None
        }
    }
}

fn seq_map_fil_find<I, Out, Map, Fil>(iter: I, map: Map, filter: Fil) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter()
        .map(map)
        .enumerate()
        .filter(|x| filter(&x.1))
        .min_by_key(|x| x.0)
}
