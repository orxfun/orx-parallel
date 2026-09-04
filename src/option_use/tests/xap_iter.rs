use crate::infallible::xap_variants::Id;
use crate::infallible_use::XapUse;
use crate::infallible_use::xap_variants::IdUse;
use crate::option_use::XapUseOptionIter;
use crate::sizes::OneOne;
use crate::use_var::UseVec;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn xap_use_option_iter_reuses_owned_use() {
    let using = UseVec::new(|_| 0usize);
    let x1 = IdUse::new(Id::<usize>::new()).map(|_, x| Some(x));
    let x2 = IdUse::new(Id::<usize>::new()).map(|u, x| {
        *u += 1;
        x + *u
    });
    let values: Vec<_> =
        XapUseOptionIter::<_, _, _, _, _, OneOne>::new(using, 0..4, x1, x2).collect();

    assert_eq!(values, vec![Some(1), Some(3), Some(5), Some(7)]);
}
