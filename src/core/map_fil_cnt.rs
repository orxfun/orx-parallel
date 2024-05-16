use super::params::RunParams;
use orx_concurrent_iter::ConcurrentIter;

pub fn map_fil_cnt<I, Out, Map, Fil>(params: RunParams, iter: I, map: Map, filter: Fil) -> usize
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
{
    let (num_threads, chunk_size) = (params.num_threads, params.chunk_size);

    std::thread::scope(|s| {
        let handles = (0..num_threads).map(|_| {
            s.spawn(|| match chunk_size {
                1 => iter.values().map(&map).filter(&filter).count(),
                _ => {
                    let mut count = 0;
                    let mut buffered = iter.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        count += chunk.values.map(&map).filter(&filter).count();
                    }
                    count
                }
            })
        });
        handles
            .map(|x| x.join().expect("Failed to join thread"))
            .sum()
    })
}
