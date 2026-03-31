use orx_parallel::*;
use std::collections::HashMap;

const N: usize = 1_000_000;

/// We create a mutable `slice` as our source: `&mut [T]`.
///
/// We can call `slice.into_par()` to create a parallel iterator
/// yielding mutable references consuming the `slice`.
///
/// `into_par` consumes the input source which is the mutable slice;
/// the `vec` is not consumed. It is mutated in place.
fn mut_slice_into_par() {
    let mut vec: Vec<_> = (0..N).collect();
    let slice = vec.as_mut_slice();

    let par = slice.into_par(); // IntoParIter on &mut [T]
    par.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().copied().reduce(|a, b| a + b).unwrap_or(0);
    assert_eq!(sum, 42);

    assert_eq!(vec.iter().sum::<usize>(), 42);
}

/// Here we directly call `par_mut` on the input `vec` since `Vec`
/// implements `ParColMut`.
///
/// `par_mut` call is non-consuming.
///
/// The `vec` is not consumed. It is mutated in place.
fn vec_par_mut() {
    let mut vec: Vec<_> = (0..N).collect();

    let par = vec.par_mut();
    par.filter(|x| **x != 42).for_each(|x| *x *= 0);

    let sum = vec.par().copied().reduce(|a, b| a + b).unwrap_or(0);
    assert_eq!(sum, 42);

    assert_eq!(vec.iter().sum::<usize>(), 42);
}

// fn iter_mut_into_par() {
//     // Finally, here we convert any mutable iterator into a parallel iterator.
//     //
//     // `Iterator<Item = &mut T>` implements `IterIntoConcurrentIter<Item = &mut T>`.
//     // => `Iterator<Item = &mut T>` auto-implements `IterIntoParIter<Item = &mut T>`
//     // Therefore, we can call `iter.iter_into_par()` to create a parallel
//     // iterator yielding mutable references.

//     let mut map: HashMap<_, _> = (0..N).map(|x| (10 * x, x)).collect();
//     let iter = map.values_mut();

//     let par = iter.iter_into_par();
//     par.filter(|x| **x != 42).for_each(|x| *x *= 0);

//     let sum = map.values().iter_into_par().sum();
//     assert_eq!(sum, 42);
// }

fn main() {
    mut_slice_into_par();
    vec_par_mut();
    // iter_mut_into_par();
}
