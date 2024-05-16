use crate::par::Par;
use orx_concurrent_iter::ConcurrentIter;

pub(crate) fn map_reduce<Data, Out, Map, Reduce>(
    par: Par<Data>,
    map: Map,
    reduce: Reduce,
) -> Option<Out>
where
    Data: ConcurrentIter,
    Data::Item: Send,
    Out: Send,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Reduce: Fn(Out, Out) -> Out + Send + Sync,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let reduce = &reduce;
    let map = &map;
    let inputs = &par.into_data();
    std::thread::scope(|s| {
        let handles = (0..num_threads).map(|_| {
            s.spawn(move || match inputs.next() {
                None => None,
                Some(first) => {
                    let acc = inputs.fold(chunk_size, map(first), |x, y| reduce(x, map(y)));
                    Some(acc)
                }
            })
        });

        let results = handles.flat_map(|x| x.join().expect("Failed to join thread"));
        results.reduce(reduce)
    })
}
