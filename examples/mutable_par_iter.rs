use orx_parallel::{IntoParIter, ParIter, ParallelizableCollection, ParallelizableCollectionMut};

const N: usize = 1_000_000;

fn using_into_par() {
    // We create a mutable `slice` as our source: `&mut [T]`
    // `&mut [T]` implements `IntoConcurrentIter<Item = &mut T>`
    // => `&mut [T]` auto-implements `IntoParIter<Item = &mut T>`
    // Therefore, we can call `slice.into_par()` to create a parallel
    // iterator yielding mutable references consuming the `slice`.

    let mut vec: Vec<_> = (0..N).collect();
    let slice = vec.as_mut_slice();
    let iter = slice.into_par(); // IntoParIter on &mut [T]
    iter.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().sum();
    assert_eq!(sum, 42);
}

fn using_par_mut() {
    // Here, we directly use the `vec` as our source: `Vec<T>`
    // `Vec<T>` implements `ConcurrentCollectionMut<Item = T>`
    // => `Vec<T>` auto-implements `ParallelizableCollectionMut<Item = T>`
    // Therefore, we can call `vec.par_mut()` to create a parallel
    // iterator yielding mutable references, using a mutable reference to `vec`.

    let mut vec: Vec<_> = (0..N).collect();
    let iter = vec.par_mut();
    iter.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().sum();
    assert_eq!(sum, 42);
}

fn main() {
    using_into_par();
    using_par_mut();
}
