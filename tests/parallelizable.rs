use orx_iterable::IntoCloningIterable;
use orx_parallel::Parallelizable;

fn take_parallelizable<T>(a: impl Parallelizable<Item = T>) {
    let _ = a.par();
    let _ = a.par();
}

#[test]
fn vec_parallelizable() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    take_parallelizable::<&String>(&vec);
}

#[test]
fn slice_parallelizable() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    take_parallelizable::<&String>(slice);
}

#[test]
fn range_parallelizable() {
    let range = 0..10;
    take_parallelizable::<usize>(range);
}

#[test]
fn cloning_iter_parallelizable() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    let iter = vec.iter().filter(|x| x.as_str() != "x");
    let cloning = iter.into_iterable();
    take_parallelizable(cloning);
}
