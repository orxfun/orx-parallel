use crate::infallible::xap_variants::Id;
use crate::infallible_use::XapUse;
use crate::infallible_use::xap_variants::IdUse;
use crate::result_use::XapUseResultIter;
use crate::sizes::OneOne;
use crate::use_var::UseVec;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn xap_use_result_iter_reuses_owned_use() {
    let using = UseVec::new(|_| 0usize);
    let x1 = IdUse::new(Id::<usize>::new()).map(|_, x| Ok::<_, ()>(x));
    let x2 = IdUse::new(Id::<usize>::new()).map(|u, x| {
        *u += 1;
        x + *u
    });
    let values: Vec<_> =
        XapUseResultIter::<_, _, _, _, _, _, OneOne>::new(using, 0..4, x1, x2).collect();

    assert_eq!(values, vec![Ok(1), Ok(3), Ok(5), Ok(7)]);
}
