use crate::IntoParIter;
use crate::infallible::xap_variants::Id;
use crate::infallible::{Xap, XapIter};
use alloc::vec::Vec;

#[test]
fn xap_iter_matches_one_to_one_flat_map() {
    let xap = Id::<usize>::new();

    let values: Vec<_> = XapIter::new(0..4, xap).collect();
    let expected: Vec<_> = (0..4).flat_map(|x| xap.xap(x)).collect();

    assert_eq!(values, expected);

    let values: Vec<_> = (0..4).into_par().into_iter().collect();
    assert_eq!(values, expected);
}

#[test]
fn xap_iter_matches_zero_or_one_flat_map() {
    let xap = Id::<usize>::new().filter_map(|x| (x % 2 == 0).then_some(x + 10));

    let values: Vec<_> = XapIter::new(0..5, xap).collect();
    let expected: Vec<_> = (0..5).flat_map(|x| xap.xap(x)).collect();

    assert_eq!(values, expected);
}

#[test]
fn xap_iter_matches_many_flat_map() {
    let xap = Id::<usize>::new().flat_map(|x| [x, x + 10]);

    let values: Vec<_> = XapIter::new(0..4, xap).collect();
    let expected: Vec<_> = (0..4).flat_map(|x| xap.xap(x)).collect();

    assert_eq!(values, expected);
}
