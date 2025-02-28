use orx_parallel::IntoPar;

fn take_into_par<T>(a: impl IntoPar<ParItem = T>) {
    let _ = a.into_par();
}

#[test]
fn vec_into_par() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    take_into_par::<String>(vec);
}

#[test]
fn slice_into_par() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    take_into_par::<&String>(slice);
}

#[test]
fn range_into_par() {
    let range = 0..10;
    take_into_par::<usize>(range);
}

#[test]
fn iter_into_par() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    let iter = vec.iter().filter(|x| x.as_str() != "x");

    // take_into_par::<&String>(iter);
}
