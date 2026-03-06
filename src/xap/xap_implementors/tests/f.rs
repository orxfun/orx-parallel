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

#[test]
fn m_f() {
    let m = |x: usize| x * 3;
    let f = |x: &usize| x.is_multiple_of(2);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().map(m).filter(f).collect();

    let xap = Id::new().map(m).filter(f);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn f_f() {
    let f1 = |x: &usize| x.is_multiple_of(2);
    let f2 = |x: &usize| x > &4;

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter(f1).filter(f2).collect();

    let xap = Id::new().filter(f1).filter(f2);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn film_f() {
    let film = |x: usize| (x > 4).then_some(x);
    let f = |x: &usize| x.is_multiple_of(2);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter_map(film).filter(f).collect();

    let xap = Id::new().filter_map(film).filter(f);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, inputs);
}
