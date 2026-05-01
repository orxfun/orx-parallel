use crate::*;
use alloc::vec;

#[test]
fn extend_par() {
    let mut vec = vec![42];
    let par = (0..170).par().map(|x| x * 2).filter(|x| *x < 50);
    vec.par_extend(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(vec, expected);
}

#[test]
fn extend_par_use() {
    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .using_clone('x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    vec.par_extend(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(vec, expected);
}
