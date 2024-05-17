use super::run_params::RunParams;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_cnt<I, Out, Map, Fil>(params: RunParams, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    match params.is_sequential() {
        true => seq_map_fil_cnt(iter, map, filter),
        _ => par_map_fil_cnt(params, iter, map, filter),
    }
}

fn par_map_fil_cnt<I, Out, Map, Fil>(params: RunParams, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    std::thread::scope(|s| {
        let mut threads = vec![];
        while params.do_spawn_new(threads.len(), iter.has_more()) {
            threads.push(s.spawn(|| match params.chunk_size() {
                1 => iter.values().map(&map).filter(&filter).count(),
                chunk_size => {
                    let mut count = 0;
                    let mut buffered = iter.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        count += chunk.values.map(&map).filter(&filter).count();
                    }
                    count
                }
            }));
        }

        threads
            .into_iter()
            .map(|x| x.join().expect("Failed to join thread"))
            .sum()
    })
}

fn seq_map_fil_cnt<I, Out, Map, Fil>(iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    iter.map(map).filter(filter).count()
}
