use crate::parallelizable_collection::ParallelizableCollection;

fn take_parallelizable_collection<T>(a: impl ParallelizableCollection<ParItem = T>) {
    let _ = a.par();
    let _ = a.par();
    let _ = a.into_par();
}
