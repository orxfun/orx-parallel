use crate::xap::{xap_implementors::id::Id, xap_trait::Xap};
use alloc::vec::Vec;
use orx_iterable::{Collection, Iterable};

fn inputs(len: usize) -> Vec<usize> {
    (0..len).collect()
}

#[test]
fn id_flam() {
    let m = |x: usize| [3 * x + 1, 4 * x];

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().flat_map(m).collect();

    let xap = Id::new().flat_map(m);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn m_flam() {
    let m1 = |x: usize| x * 3;
    let m = |x: usize| [3 * x + 1, 4 * x];

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().map(m1).flat_map(m).collect();

    let xap = Id::new().map(m1).flat_map(m);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn f_flam() {
    let f = |x: &usize| x.is_multiple_of(2);
    let m = |x: usize| [3 * x + 1, 4 * x];

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter(f).flat_map(m).collect();

    let xap = Id::new().filter(f).flat_map(m);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn film_flam() {
    let film = |x: usize| (x > 4).then_some(x);
    let m = |x: usize| [3 * x + 1, 4 * x];

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().filter_map(film).flat_map(m).collect();

    let xap = Id::new().filter_map(film).flat_map(m);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}

#[test]
fn flam_flam() {
    let flam = |x: usize| [x + 3, x + 6];
    let m = |x: usize| [3 * x + 1, 4 * x];

    let inputs = inputs(10);
    let copied = inputs.as_iterable().copied();
    let expected: Vec<_> = copied.iter().flat_map(flam).flat_map(m).collect();

    let xap = Id::new().flat_map(flam).flat_map(m);
    let values: Vec<_> = copied.iter().flat_map(|i| xap.xap(i)).collect();

    assert_eq!(expected, values);
}
