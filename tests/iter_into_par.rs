use orx_concurrent_iter::IterIntoConcurrentIter;
use orx_parallel::IterIntoParIter;

fn take_iter_into_par_iterator_bounds<I>(a: I)
where
    I: Iterator,
    I::Item: Send + Sync,
{
    let _par_iter = a.iter_into_par();
}

fn take_iter_into_par_iter_into_con_iter_bounds<I>(a: I)
where
    I: IterIntoConcurrentIter,
    I::Item: Send + Sync,
{
    let _par_iter = a.iter_into_par();
}

fn take_iter_into_par_iter_into_par_bounds<T>(a: impl IterIntoParIter<Item = T>) {
    let _ = a.iter_into_par();
}

#[test]
fn iter_into_par_iter() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();

    let iter = vec.iter().filter(|x| x.as_str() != "x");
    take_iter_into_par_iterator_bounds(iter);

    let iter = vec.iter().filter(|x| x.as_str() != "x");
    take_iter_into_par_iter_into_con_iter_bounds(iter);

    let iter = vec.iter().filter(|x| x.as_str() != "x");
    take_iter_into_par_iter_into_par_bounds(iter);
}
