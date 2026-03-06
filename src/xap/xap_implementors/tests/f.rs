use crate::xap::{xap_implementors::id::Id, xap_trait::Xap};
use alloc::vec::Vec;
use orx_iterable::{Collection, Iterable};

fn inputs(len: usize) -> Vec<usize> {
    (0..len).collect()
}

#[test]
fn id_f() {
    let f = |x: &usize| x.is_multiple_of(2);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter(f).collect();

    let xap = Id::new().filter(f);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}
