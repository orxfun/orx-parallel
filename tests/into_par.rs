use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::IntoParIter;

fn take_into_par_into_par_bounds<T>(a: impl IntoParIter<Item = T>) {
    let _ = a.into_par();
}

fn take_into_par_into_con_iter_bounds<T>(a: impl IntoConcurrentIter<Item = T>) {
    let _ = a.into_par();
}

#[test]
fn vec_into_par_iter() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();

    take_into_par_into_par_bounds::<String>(vec.clone());

    take_into_par_into_con_iter_bounds::<String>(vec);
}

#[test]
fn slice_into_par_iter() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();

    take_into_par_into_par_bounds::<&String>(slice);

    take_into_par_into_con_iter_bounds::<&String>(slice);
}

#[test]
fn range_into_par_iter() {
    let range = 0..10;

    take_into_par_into_par_bounds::<usize>(range.clone());

    take_into_par_into_con_iter_bounds::<usize>(range);
}
