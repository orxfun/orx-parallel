use super::diagnostics::ParThreadLogger;
use super::runner::{ParTask, Runner};
use crate::Params;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::PinnedVec;

pub fn par_fmap_fil_colx<I, OutIter, Out, Map, Fil, P>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
) -> ConcurrentBag<Out, P>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    #[cfg(feature = "with_diagnostics")]
    par_fmap_fil_colx_core::<_, _, _, _, _, _, super::diagnostics::ParLogger>(
        params, iter, map, filter, &collected,
    );

    #[cfg(not(feature = "with_diagnostics"))]
    par_fmap_fil_colx_core::<_, _, _, _, _, _, super::diagnostics::NoLogger>(
        params, iter, map, filter, &collected,
    );

    collected
}

fn par_fmap_fil_colx_core<I, OutIter, Out, Map, Fil, P, L>(
    params: Params,
    iter: I,
    map: Map,
    filter: Fil,
    collected: &ConcurrentBag<Out, P>,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    L: ParThreadLogger,
{
    let task = |c| task::<_, _, _, _, _, _, L>(&iter, &map, &filter, collected, c);
    let num_spawned = Runner::run(params, ParTask::Collect, &iter, &task);

    L::log_num_spawned(num_spawned);
}

fn task<I, OutIter, Out, Map, Fil, P, L>(
    iter: &I,
    map: &Map,
    filter: &Fil,
    collected: &ConcurrentBag<Out, P>,
    chunk_size: usize,
) where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    L: ParThreadLogger,
{
    let logger = L::new(chunk_size);
    match chunk_size {
        1 => {
            while let Some(x) = iter.next_id_and_value() {
                for x in map(x.value).into_iter().filter(filter) {
                    collected.push(x);
                }
            }
        }
        c => {
            let mut buffer = vec![];
            let mut buffered = iter.buffered_iter(c);
            while let Some(chunk) = buffered.next() {
                logger.next_chunk(chunk.values.len());
                assert!(buffer.is_empty());
                buffer.extend(chunk.values.flat_map(map).filter(filter));
                collected.extend(buffer.drain(0..buffer.len()));

                // let buffer: Vec<_> = chunk.values.flat_map(map).filter(filter).collect();
                // collected.extend(buffer);
            }
        }
    }
}

pub fn seq_fmap_fil_colx<I, OutIter, Out, Map, Fil, P>(
    iter: I,
    map: Map,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
) -> ConcurrentBag<Out, P>
where
    I: ConcurrentIter,
    OutIter: IntoIterator<Item = Out>,
    Out: Default + Send + Sync,
    Map: Fn(I::Item) -> OutIter + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
{
    let mut vec = collected.into_inner();
    let iter = iter.into_seq_iter();
    for x in iter.flat_map(map).filter(filter) {
        vec.push(x);
    }
    vec.into()
}
