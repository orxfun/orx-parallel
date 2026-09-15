use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::result::XapResultIter;
use crate::sizes::OneOne;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn xap_result_iter_matches_successful_pipeline() {
    let x1 = Id::<usize>::new().map(Ok::<_, ()>);
    let x2 = Id::<usize>::new();
    let values: Vec<_> = XapResultIter::<_, _, _, _, _, OneOne>::new(0..4, x1, x2).collect();

    assert_eq!(values, vec![Ok(0), Ok(1), Ok(2), Ok(3)]);
}

#[test]
fn xap_result_iter_preserves_errors() {
    let x1 = Id::<usize>::new().map(|x| if x == 2 { Err("error") } else { Ok(x) });
    let x2 = Id::<usize>::new();
    let values: Vec<_> = XapResultIter::<_, _, _, _, _, OneOne>::new(0..4, x1, x2).collect();

    assert_eq!(values, vec![Ok(0), Ok(1), Err("error"), Ok(3)]);
}
