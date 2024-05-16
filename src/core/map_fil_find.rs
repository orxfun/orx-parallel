use super::params::RunParams;
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
    let (num_threads, chunk_size) = (params.num_threads, params.chunk_size);

    std::thread::scope(|s| {
        let results_threads = (0..(num_threads - 1))
            .map(|_| {
                s.spawn(|| match chunk_size {
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
                    _ => {
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
                })
            })
            .collect::<Vec<_>>();

        let mut result_main = None;
        match chunk_size {
            1 => {
                let result = iter
                    .ids_and_values()
                    .map(|x| (x.0, map(x.1)))
                    .find(|x| filter(&x.1));
                if result.is_some() {
                    iter.skip_to_end();
                    result_main = result;
                }
            }
            _ => {
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
                        result_main = result;
                        break;
                    }
                }
            }
        };

        let mut results = Vec::with_capacity(num_threads);
        for x in results_threads {
            if let Some(x) = x.join().expect("Failed to join thread") {
                results.push(x);
            }
        }
        if let Some(x) = result_main {
            results.push(x);
        }

        results.into_iter().min_by_key(|x| x.0)

        // (0..(num_threads))
        //     .map(|_| {
        //         s.spawn(|| match chunk_size {
        //             1 => {
        //                 for (i, x) in iter.ids_and_values() {
        //                     let value = map(x);
        //                     if filter(&value) {
        //                         iter.skip_to_end();
        //                         return Some((i, value));
        //                     }
        //                 }
        //                 None
        //             }
        //             _ => {
        //                 let mut buffered = iter.buffered_iter(chunk_size);
        //                 while let Some(chunk) = buffered.next() {
        //                     for (i, x) in chunk.values.enumerate() {
        //                         let value = map(x);
        //                         if filter(&value) {
        //                             iter.skip_to_end();
        //                             return Some((chunk.begin_idx + i, value));
        //                         }
        //                     }
        //                 }
        //                 None
        //             }
        //         })
        //     })
        //     .collect::<Vec<_>>()
        //     .into_iter()
        //     .flat_map(|x| x.join().expect("Failed to join thread"))
        //     .min_by_key(|x| x.0)
    })
}
