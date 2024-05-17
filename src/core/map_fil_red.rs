use std::fmt::Debug;

use super::run_params::RunParams;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_red<I, Out, Map, Fil, Red>(
    params: RunParams,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync + Debug,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_red(iter, map, filter, reduce),
        _ => par_map_fil_red(params, iter, map, filter, reduce),
    }
}

fn par_map_fil_red<I, Out, Map, Fil, Red>(
    params: RunParams,
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync + Debug,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    std::thread::scope(|s| {
        let mut threads = vec![];
        while params.do_spawn_new(threads.len(), iter.has_more()) {
            threads.push(s.spawn(|| match params.chunk_size() {
                1 => {
                    while let Some(next) = iter.next() {
                        let first = map(next);
                        if filter(&first) {
                            let acc = iter.values().map(&map).filter(&filter).fold(first, &reduce);
                            return Some(acc);
                        }
                    }
                    None
                }
                chunk_size => {
                    while let Some(next) = iter.next() {
                        let first = map(next);
                        if filter(&first) {
                            let mut acc = first;
                            let mut buffered = iter.buffered_iter(chunk_size);
                            while let Some(chunk) = buffered.next() {
                                if let Some(chunk_acc) =
                                    chunk.values.map(&map).filter(&filter).reduce(&reduce)
                                {
                                    acc = reduce(acc, chunk_acc);
                                }
                            }
                            return Some(acc);
                        }
                    }
                    None
                }
            }));
        }

        // dbg!(threads.len(), params);
        // assert_eq!(threads.len(), 33);

        threads
            .into_iter()
            .flat_map(|x| x.join().expect("Failed to join thread"))
            .reduce(&reduce)
    })
}

fn seq_map_fil_red<I, Out, Map, Fil, Red>(
    iter: I,
    map: Map,
    filter: Fil,
    reduce: Red,
) -> Option<Out>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    let iter = iter.into_seq_iter();
    iter.map(map).filter(filter).reduce(reduce)
}
