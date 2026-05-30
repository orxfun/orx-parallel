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
        .use_new(|_| 'x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    vec.par_extend(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(vec, expected);
}

#[test]
fn extend_par_opt() {
    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(Some)
        .into_optional()
        .map(|x| x * 2)
        .filter(|x| *x < 50);
    let ok = vec.par_extend_opt(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(ok, Some(()));
    assert_eq!(vec, expected);

    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(|x| (x < 10).then_some(x))
        .into_optional()
        .map(|x| x * 2)
        .filter(|x| *x < 50);
    let ok = vec.par_extend_opt(par);
    assert_eq!(ok, None);
}

#[test]
fn extend_par_use_opt() {
    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(Some)
        .into_optional()
        .use_new(|_| 'x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    let ok = vec.par_extend_opt(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(ok, Some(()));
    assert_eq!(vec, expected);

    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(|x| (x < 10).then_some(x))
        .into_optional()
        .use_new(|_| 'x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    let ok = vec.par_extend_opt(par);
    assert_eq!(ok, None);
}

#[test]
fn extend_par_res() {
    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(Result::<_, char>::Ok)
        .into_fallible()
        .map(|x| x * 2)
        .filter(|x| *x < 50);
    let ok = vec.par_extend_res(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(ok, Ok(()));
    assert_eq!(vec, expected);

    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(|x| match x < 10 {
            true => Ok(x),
            false => Err('x'),
        })
        .into_fallible()
        .map(|x| x * 2)
        .filter(|x| *x < 50);
    let ok = vec.par_extend_res(par);
    assert_eq!(ok, Err('x'));
}

#[test]
fn extend_par_use_res() {
    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(Result::<_, char>::Ok)
        .into_fallible()
        .use_new(|_| 'x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    let ok = vec.par_extend_res(par);

    let mut expected = vec![42];
    let iter = (0..170).into_iter().map(|x| x * 2).filter(|x| *x < 50);
    expected.extend(iter);

    assert_eq!(ok, Ok(()));
    assert_eq!(vec, expected);

    let mut vec = vec![42];
    let par = (0..170)
        .par()
        .map(|x| match x < 10 {
            true => Ok(x),
            false => Err('x'),
        })
        .into_fallible()
        .use_new(|_| 'x')
        .map(|_, x| x * 2)
        .filter(|_, x| *x < 50);
    let ok = vec.par_extend_res(par);
    assert_eq!(ok, Err('x'));
}
