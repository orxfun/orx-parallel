use orx_parallel::*;

fn take_parallelizable_collection<T>(a: impl ParCollection<Item = T>) {
    let _ = a.par();
    let _ = a.par();
}

fn take_parallelizable_collection_mut<T>(mut a: impl ParCollectionMut<Item = T>) {
    let _ = a.par_mut();
    let _ = a.par_mut();
}

#[test]
fn vec_parallelizable_collection() {
    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    take_parallelizable_collection::<String>(vec);

    let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
    take_parallelizable_collection_mut::<String>(vec);
}
