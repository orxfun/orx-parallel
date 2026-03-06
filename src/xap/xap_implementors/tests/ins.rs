use crate::xap::{xap_implementors::id::Id, xap_trait::Xap};
use alloc::vec::Vec;
use orx_iterable::{Collection, Iterable};
use std::println;

fn inputs(len: usize) -> Vec<usize> {
    (0..len).collect()
}

#[test]
fn id_ins() {
    let ins = |x: &usize| println!("{}", 3 * x + 1);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().inspect(ins).collect();

    let xap = Id::new().inspect(ins);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn m_ins() {
    let m1 = |x: usize| x * 3;
    let ins = |x: &usize| println!("{}", 3 * x + 1);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().map(m1).inspect(ins).collect();

    let xap = Id::new().map(m1).inspect(ins);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn f_ins() {
    let f = |x: &usize| x.is_multiple_of(2);
    let ins = |x: &usize| println!("{}", 3 * x + 1);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter(f).inspect(ins).collect();

    let xap = Id::new().filter(f).inspect(ins);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn film_ins() {
    let film = |x: usize| (x > 4).then_some(x);
    let ins = |x: &usize| println!("{}", 3 * x + 1);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter_map(film).inspect(ins).collect();

    let xap = Id::new().filter_map(film).inspect(ins);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn flam_ins() {
    let flam = |x: usize| [x + 3, x + 6];
    let ins = |x: &usize| println!("{}", 3 * x + 1);

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().flat_map(flam).inspect(ins).collect();

    let xap = Id::new().flat_map(flam).inspect(ins);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}
