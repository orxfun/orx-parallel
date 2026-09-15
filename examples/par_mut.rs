use orx_parallel::*;
use std::collections::HashMap;

const DISCOUNT_THRESHOLD: usize = 1_000;
const DISCOUNT_CENTS: usize = 200;

fn apply_discount(price: &mut usize) {
    *price -= DISCOUNT_CENTS;
}

/// A mutable slice, `&mut [T]`, can be turned into a parallel iterator with
/// `into_par()`. The slice handle is consumed, while the backing vector is
/// mutated in place.
fn mut_slice_into_par() {
    let mut vec = vec![750, 1_200, 2_500, 900, 1_800];
    let slice = vec.as_mut_slice();

    slice
        .into_par()
        .filter(|price| **price >= DISCOUNT_THRESHOLD)
        .for_each(apply_discount);

    assert_eq!(vec, vec![750, 1_000, 2_300, 900, 1_600]);
}

/// `Vec<T>` provides the more direct `par_mut()` constructor. This call is
/// non-consuming and mutates the vector in place.
fn vec_par_mut() {
    let mut vec = vec![750, 1_200, 2_500, 900, 1_800];

    vec.par_mut()
        .filter(|price| **price >= DISCOUNT_THRESHOLD)
        .for_each(apply_discount);

    assert_eq!(vec, vec![750, 1_000, 2_300, 900, 1_600]);
}

/// Any mutable iterator can also be parallelized. Here we convert
/// `map.values_mut()` into a parallel iterator of mutable references.
fn iter_mut_into_par() {
    let mut map: HashMap<_, _> = HashMap::from([
        ("pen", 750),
        ("headphones", 1_200),
        ("keyboard", 2_500),
        ("notebook", 900),
        ("monitor", 1_800),
    ]);

    map.values_mut()
        .iter_into_par()
        .filter(|price| **price >= DISCOUNT_THRESHOLD)
        .for_each(apply_discount);

    assert_eq!(map.values().sum::<usize>(), 6_550);
}

fn main() {
    mut_slice_into_par();
    vec_par_mut();
    iter_mut_into_par();
}
