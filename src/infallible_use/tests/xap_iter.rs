use crate::infallible::xap_variants::Id;
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{XapUse, XapUseIter};
use crate::use_var::{Use, UseVec};
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn xap_use_iter_matches_one_to_one_flat_map() {
    let using = UseVec::new(|_| 0usize);
    let xap = IdUse::new(Id::<usize>::new());

    let values: Vec<_> = XapUseIter::new(using, 0..4, xap).collect();

    let using = UseVec::new(|_| 0usize);
    // SAFETY: this sequential comparison uses worker index 0 exactly once.
    let use_ptr = unsafe { using.init_get(0) };
    let expected: Vec<_> = (0..4).flat_map(|x| xap.xap_use(use_ptr, x)).collect();

    assert_eq!(values, expected);
}

#[test]
fn xap_use_iter_reuses_owned_use_value() {
    let using = UseVec::new(|_| 0usize);
    let xap = IdUse::new(Id::<usize>::new()).map(|u, x| {
        *u += 1;
        x + *u
    });

    let values: Vec<_> = XapUseIter::new(using, 0..4, xap).collect();

    assert_eq!(values, vec![1, 3, 5, 7]);
}
