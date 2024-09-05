use super::runner::{ParTask, Runner};
use crate::core::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterX};

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
        false => par_fmap_fil_find(params, iter, fmap, filter),
    }
}

fn par_fmap_fil_find<I, OutIter, Out, Map, Fil>(
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
    let task = |c| task(&iter, &fmap, &filter, c);
    let reduce =
        |a: Option<(usize, _)>, b| maybe_reduce(|a, b| if b.0 < a.0 { b } else { a }, a, b);
    let (_num_spawned, found) = Runner::reduce(params, ParTask::EarlyReturn, &iter, &task, reduce);

    found.flatten().map(|x| x.1)
}

fn task<I, OutIter, Out, Map, Fil>(
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
{
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
    I: ConcurrentIterX,
    OutIter: IntoIterator<Item = Out>,
    Out: Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter().flat_map(map).find(filter)
}
