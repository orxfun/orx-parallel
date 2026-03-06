use crate::xap::{xap_implementors::id::Id, xap_trait::Xap};
use alloc::vec::Vec;

fn inputs(len: usize) -> Vec<usize> {
    (0..len).collect()
}

#[test]
fn id() {
    let inputs = inputs(10);
    let xap = Id::new();
    let values: Vec<_> = inputs.iter().copied().flat_map(|i| xap.xap(i)).collect();
    assert_eq!(inputs, values);
}
