use orx_parallel::{
    IntoParIter, IterIntoParIter, ParIter, ParallelizableCollection, ParallelizableCollectionMut,
};
use std::collections::HashMap;

const N: usize = 1_000_000;

fn mut_slice_into_par() {
    // We create a mutable `slice` as our source: `&mut [T]`
    //
    // `&mut [T]` implements `IntoConcurrentIter<Item = &mut T>`
    // => `&mut [T]` auto-implements `IntoParIter<Item = &mut T>`
    // Therefore, we can call `slice.into_par()` to create a parallel
    // iterator yielding mutable references consuming the `slice`.

    let mut vec: Vec<_> = (0..N).collect();
    let slice = vec.as_mut_slice();

    let par = slice.into_par(); // IntoParIter on &mut [T]
    par.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().sum();
    assert_eq!(sum, 42);
}

fn vec_par_mut() {
    // Here, we directly use the `vec` as our source: `Vec<T>`
    //
    // `Vec<T>` implements `ConcurrentCollectionMut<Item = T>`
    // => `Vec<T>` auto-implements `ParallelizableCollectionMut<Item = T>`
    // Therefore, we can call `vec.par_mut()` to create a parallel
    // iterator yielding mutable references, using a mutable reference to `vec`.

    let mut vec: Vec<_> = (0..N).collect();

    let par = vec.par_mut();
    par.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().sum();
    assert_eq!(sum, 42);
}

fn iter_mut_into_par() {
    // Finally, here we convert any mutable iterator into a parallel iterator.
    //
    // `Iterator<Item = &mut T>` implements `IterIntoConcurrentIter<Item = &mut T>`.
    // => `Iterator<Item = &mut T>` auto-implements `IterIntoParIter<Item = &mut T>`
    // Therefore, we can call `iter.iter_into_par()` to create a parallel
    // iterator yielding mutable references.

    let mut map: HashMap<_, _> = (0..N).map(|x| (10 * x, x)).collect();
    let iter = map.values_mut();

    let par = iter.iter_into_par();
    par.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = map.values().iter_into_par().sum();
    assert_eq!(sum, 42);
}

fn main() {
    mut_slice_into_par();
    vec_par_mut();
    iter_mut_into_par();
}
