use super::run_params::RunParams;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_find<I, Out, Map, Fil>(
    params: RunParams,
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
        _ => par_map_fil_find(params, iter, map, filter),
    }
}

fn par_map_fil_find<I, Out, Map, Fil>(
    params: RunParams,
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
    std::thread::scope(|s| {
        let mut threads = vec![];
        while params.do_spawn_new(threads.len(), iter.has_more()) {
            threads.push(s.spawn(|| match params.chunk_size() {
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
                chunk_size => {
                    let mut buffered = iter.buffered_iter(chunk_size);
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
            }));
        }

        threads
            .into_iter()
            .flat_map(|x| x.join().expect("Failed to join thread"))
            .min_by_key(|x| x.0)
    })
}

fn seq_map_fil_find<I, Out, Map, Fil>(iter: I, map: Map, filter: Fil) -> Option<(usize, Out)>
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let iter = iter.into_seq_iter();
    iter.map(map)
        .enumerate()
        .filter(|x| filter(&x.1))
        .min_by_key(|x| x.0)
}
