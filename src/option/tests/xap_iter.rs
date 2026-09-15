use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::option::XapOptionIter;
use crate::sizes::OneOne;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn xap_option_iter_matches_successful_pipeline() {
    let x1 = Id::<usize>::new().map(Some);
    let x2 = Id::<usize>::new();
    let values: Vec<_> = XapOptionIter::<_, _, _, _, OneOne>::new(0..4, x1, x2).collect();

    assert_eq!(values, vec![Some(0), Some(1), Some(2), Some(3)]);
}

#[test]
fn xap_option_iter_short_circuits_with_none() {
    let x1 = Id::<usize>::new().map(|x| (x != 2).then_some(x));
    let x2 = Id::<usize>::new();
    let values: Vec<_> = XapOptionIter::<_, _, _, _, OneOne>::new(0..4, x1, x2).collect();

    assert_eq!(values, vec![Some(0), Some(1), None, Some(3)]);
}
