use super::runner::{ParTask, Runner};
use crate::core::utils::maybe_reduce;
use crate::{Fallible, Params};
use orx_concurrent_iter::ConcurrentIter;

pub fn filtermap_fil_find<I, FO, Out, FilterMap, Fil>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_filtermap_fil_find(iter, filter_map, filter),
        false => par_filtermap_fil_find(params, iter, filter_map, filter),
    }
}

fn par_filtermap_fil_find<I, FO, Out, FilterMap, Fil>(
    params: Params,
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let task = |c| task(&iter, &filter_map, &filter, c);
    let reduce =
        |a: Option<(usize, _)>, b| maybe_reduce(|a, b| if b.0 < a.0 { b } else { a }, a, b);
    let (_num_spawned, found) = Runner::reduce(params, ParTask::EarlyReturn, &iter, &task, reduce);

    found.flatten()
}

fn task<I, FO, Out, FilterMap, Fil>(
    iter: &I,
    filter_map: &FilterMap,
    filter: &Fil,
    chunk_size: usize,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match chunk_size {
        1 => {
            let result = iter
                .ids_and_values()
                .map(|x| (x.0, filter_map(x.1)))
                .find_map(|x| match x.1.has_value() {
                    false => None,
                    true => {
                        let value = x.1.value();
                        match filter(&value) {
                            false => None,
                            true => Some((x.0, value)),
                        }
                    }
                });

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
                    .map(|x| (x.0, filter_map(x.1)))
                    .find_map(|x| match x.1.has_value() {
                        false => None,
                        true => {
                            let value = x.1.value();
                            match filter(&value) {
                                false => None,
                                true => Some((chunk.begin_idx + x.0, value)),
                            }
                        }
                    });

                if result.is_some() {
                    iter.skip_to_end();
                    return result;
                }
            }
            None
        }
    }
}

fn seq_filtermap_fil_find<I, FO, Out, FilterMap, Fil>(
    iter: I,
    filter_map: FilterMap,
    filter: Fil,
) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    FO: Fallible<Out> + Send + Sync,
    Out: Send + Sync,
    FilterMap: Fn(I::Item) -> FO + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    iter.into_seq_iter()
        .map(filter_map)
        .enumerate()
        .find_map(|x| match x.1.has_value() {
            false => None,
            true => {
                let value = x.1.value();
                match filter(&value) {
                    false => None,
                    true => Some((x.0, value)),
                }
            }
        })
}
