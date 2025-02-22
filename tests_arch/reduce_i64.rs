use orx_parallel::*;
use std::cmp::Ordering;

fn some_if(condition: bool, value: i64) -> Option<i64> {
    if condition {
        Some(value)
    } else {
        None
    }
}

pub fn reduce<R: Par<Item = i64>>(par: R, len: usize) {
    fn add(a: i64, b: i64) -> i64 {
        a + b
    }
    let expected = some_if(len > 0, (0..len).sum::<usize>() as i64);
    let result = par.reduce(add);
    assert_eq!(result, expected);
}

pub fn sum<R: Par<Item = i64>>(par: R, len: usize) {
    let expected = (0..len).sum::<usize>() as i64;
    let result = par.sum();
    assert_eq!(result, expected);
}

pub fn min<R: Par<Item = i64>>(par: R, len: usize) {
    let expected = some_if(len > 0, 0);
    let result = par.min();
    assert_eq!(result, expected);
}

pub fn max<R: Par<Item = i64>>(par: R, len: usize) {
    let expected = some_if(len > 0, len as i64 - 1);
    let result = par.max();
    assert_eq!(result, expected);
}

pub fn min_by<R: Par<Item = i64>>(par: R, len: usize) {
    fn compare(a: &i64, b: &i64) -> Ordering {
        match b % 2 {
            0 => a.cmp(b),
            _ => Ordering::Less,
        }
    }
    let expected = (0..len).map(|x| x as i64).min_by(compare);
    let result = par.min_by(compare);
    assert_eq!(result, expected);
}

pub fn max_by<R: Par<Item = i64>>(par: R, len: usize) {
    fn compare(a: &i64, b: &i64) -> Ordering {
        match b % 2 {
            0 => a.cmp(b),
            _ => Ordering::Greater,
        }
    }
    let expected = (0..len).map(|x| x as i64).max_by(compare);
    let result = par.max_by(compare);

    assert_eq!(result, expected);
}

pub fn min_by_key<R: Par<Item = i64>>(par: R, len: usize) {
    fn get_key(a: &i64) -> usize {
        match a % 2 {
            0 => usize::MAX,
            _ => *a as usize,
        }
    }
    let expected = (0..len).map(|x| x as i64).min_by_key(get_key);
    let result = par.min_by_key(get_key);
    assert_eq!(result, expected);
}

pub fn max_by_key<R: Par<Item = i64>>(par: R, len: usize) {
    fn get_key(a: &i64) -> usize {
        match a % 2 {
            0 => 0,
            _ => *a as usize,
        }
    }
    let expected = (0..len).map(|x| x as i64).max_by_key(get_key);
    let result = par.max_by_key(get_key);
    assert_eq!(result, expected);
}
