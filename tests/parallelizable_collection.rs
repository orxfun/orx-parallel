// use orx_parallel::ParallelizableCollection;

// fn take_parallelizable_collection<T>(a: impl ParallelizableCollection<ParItem = T>) {
//     let _ = a.par();
//     let _ = a.par();
//     let _ = a.into_par();
// }

// #[test]
// fn vec_parallelizable_collection() {
//     let vec: Vec<_> = (0..10).map(|x| x.to_string()).collect();
//     take_parallelizable_collection::<String>(vec);
// }
