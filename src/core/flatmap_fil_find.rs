use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::core::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub fn fmap_fil_find<I, OutIter, Out, Map, Fil>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_fmap_fil_find(iter, fmap, filter),
        false => {
            #[cfg(feature = "with_diagnostics")]
            return par_fmap_fil_find::<_, _, _, _, _, super::diagnostics::ParLogger>(
                params, iter, fmap, filter,
            );

            #[cfg(not(feature = "with_diagnostics"))]
            par_fmap_fil_find::<_, _, _, _, _, super::diagnostics::NoLogger>(
                params, iter, fmap, filter,
            )
        }
    }
}

fn par_fmap_fil_find<I, OutIter, Out, Map, Fil, L>(
    params: Params,
    iter: I,
    fmap: Map,
    filter: Fil,
) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, L>(&iter, &fmap, &filter, c);
    let reduce =
        |a: Option<(usize, _)>, b| maybe_reduce(|a, b| if b.0 < a.0 { b } else { a }, a, b);
    let (num_spawned, found) = Runner::reduce(params, ParTask::EarlyReturn, &iter, &task, reduce);

    L::log_num_spawned(num_spawned);
    found.flatten().map(|x| x.1)
}

fn task<I, OutIter, Out, Map, Fil, L>(
    iter: &I,
    fmap: &Map,
    filter: &Fil,
    chunk_size: usize,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            let result = iter
                .ids_and_values()
                .flat_map(|x| fmap(x.1).into_iter().find(filter).map(|y| (x.0, y)))
                .next();

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
                    .flat_map(|x| {
                        fmap(x.1)
                            .into_iter()
                            .find(filter)
                            .map(|y| (chunk.begin_idx + x.0, y))
                    })
                    .next();

                if result.is_some() {
                    iter.skip_to_end();
                    return result;
                }
            }

            None
        }
    }
}

fn seq_fmap_fil_find<I, OutIter, Out, Map, Fil>(iter: I, map: Map, filter: Fil) -> Option<Out>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter().flat_map(map).find(filter)
}
