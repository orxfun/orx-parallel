use crate::par::Par;
use orx_concurrent_iter::ConcurrentIter;

pub(crate) fn reduce<Data, Reduce>(par: Par<Data>, reduce: Reduce) -> Option<Data::Item>
where
    Data: ConcurrentIter,
    Data::Item: Send,
    Reduce: Fn(Data::Item, Data::Item) -> Data::Item + Send + Sync,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let reduce = &reduce;
    let inputs = &par.into_data();
    std::thread::scope(|s| {
        let handles = (0..num_threads).map(|_| {
            s.spawn(move || {
                inputs
                    .next()
                    .map(|first| inputs.fold(chunk_size, first, reduce))
            })
        });

        let results = handles.flat_map(|x| x.join().expect("Failed to join thread"));
        results.reduce(reduce)
    })
}
