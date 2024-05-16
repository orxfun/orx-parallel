use super::params::RunParams;
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
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    Red: Fn(Out, Out) -> Out + Send + Sync,
{
    let (num_threads, mut chunk_size) = (params.num_threads, params.chunk_size);

    if let Some(len) = iter.try_get_len() {
        chunk_size = len / num_threads;
    }

    std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| {
                s.spawn(|| match chunk_size {
                    1 => {
                        while let Some(next) = iter.next() {
                            let first = map(next);
                            if filter(&first) {
                                let acc =
                                    iter.values().map(&map).filter(&filter).fold(first, &reduce);
                                return Some(acc);
                            }
                        }
                        None
                    }
                    _ => {
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
                                        return Some(acc);
                                    }
                                }
                            }
                        }
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|x| x.join().expect("Failed to join thread"))
            .reduce(&reduce)
    })
}
