use super::runner::{ParTask, Runner};
use crate::core::utils::maybe_reduce;
use crate::Params;
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterX};

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
        false => par_map_fil_find(params, iter, map, filter),
    }
}

fn par_map_fil_find<I, Out, Map, Fil>(
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
    let task = |c| task(&iter, &map, &filter, c);
    let reduce =
        |a: Option<(usize, _)>, b| maybe_reduce(|a, b| if b.0 < a.0 { b } else { a }, a, b);
    let (_num_spawned, found) = Runner::reduce(params, ParTask::EarlyReturn, &iter, &task, reduce);

    found.flatten()
}

fn task<I, Out, Map, Fil>(
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
{
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
    I: ConcurrentIterX,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter()
        .map(map)
        .enumerate()
        .find_map(|x| match filter(&x.1) {
            false => None,
            true => Some(x),
        })
}
