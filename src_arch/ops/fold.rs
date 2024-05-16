use crate::par::Par;
use orx_concurrent_iter::ConcurrentIter;

pub(crate) fn fold<Data, T, Fold, Reduce>(
    par: Par<Data>,
    neutral: T,
    fold: Fold,
    reduce: Reduce,
) -> T
where
    Data: ConcurrentIter,
    T: Send + Sync + Clone,
    Fold: Fn(T, Data::Item) -> T + Send + Sync,
    Reduce: Fn(T, T) -> T,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let fold = &fold;
    let reduce = &reduce;
    let inputs = &par.into_data();
    let n = || neutral.clone();
    std::thread::scope(|s| {
        let handles = (0..num_threads).map(|_| s.spawn(move || inputs.fold(chunk_size, n(), fold)));

        let results = handles.map(|x| x.join().expect("Failed to join thread"));
        results.fold(n(), reduce)
    })
}
