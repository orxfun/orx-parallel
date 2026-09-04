use crate::*;
use alloc::vec;

#[test]
fn size_hint_one_for_exact_sized_input() {
    let input = vec![1, 3, 5, 7];
    let par = input.into_par().map(Some).into_optional().use_new(|_| ());
    assert_eq!(par.size_hint(), (4, Some(4)));
    let par = par.map(|_, x| x + 1);
    assert_eq!(par.size_hint(), (4, Some(4)));
}

#[test]
fn size_hint_one_for_unknown_sized_input() {
    let input = vec![1, 3, 5, 7]
        .into_iter()
        .filter(|x| *x > 3)
        .flat_map(|x| (0..3).map(move |y| y + x + 1));
    let par = input
        .iter_into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ());
    assert_eq!(par.size_hint(), (0, None));
    let par = par.map(|_, x| x + 1);
    assert_eq!(par.size_hint(), (0, None));
}

#[test]
fn size_hint_bin_for_exact_sized_input() {
    let input = vec![1, 3, 5, 7];
    let par = input
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ())
        .filter(|_, x| *x < 2);
    assert_eq!(par.size_hint(), (0, Some(4)));
    let par = par.filter_map(|_, x| (x < 3).then_some(6 + x));
    assert_eq!(par.size_hint(), (0, Some(4)));
}

#[test]
fn size_hint_bin_for_unknown_sized_input() {
    let input = vec![1, 3, 5, 7]
        .into_iter()
        .filter(|x| *x > 3)
        .flat_map(|x| (0..3).map(move |y| y + x + 1));
    let par = input
        .iter_into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ())
        .filter(|_, x| *x < 2);
    assert_eq!(par.size_hint(), (0, None));
    let par = par.filter_map(|_, x| (x < 3).then_some(6 + x));
    assert_eq!(par.size_hint(), (0, None));
}

#[test]
fn size_hint_many_for_exact_sized_input() {
    let input = vec![1, 3, 5, 7];
    let par = input
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ())
        .flat_map(|_, x| (0..2).map(move |y| x + y));
    assert_eq!(par.size_hint(), (0, None));
    let par = par.flat_map(|_, x| (0..2).map(move |y| x + y));
    assert_eq!(par.size_hint(), (0, None));
}

#[test]
fn size_hint_many_for_unknown_sized_input() {
    let input = vec![1, 3, 5, 7]
        .into_iter()
        .filter(|x| *x > 3)
        .flat_map(|x| (0..3).map(move |y| y + x + 1));
    let par = input
        .iter_into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ())
        .flat_map(|_, x| (0..2).map(move |y| x + y));
    assert_eq!(par.size_hint(), (0, None));
    let par = par.flat_map(|_, x| (0..2).map(move |y| x + y));
    assert_eq!(par.size_hint(), (0, None));
}
