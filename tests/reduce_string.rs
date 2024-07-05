use orx_parallel::*;
use std::cmp::Ordering;

fn some_if(condition: bool, value: String) -> Option<String> {
    if condition {
        Some(value)
    } else {
        None
    }
}

pub fn reduce<R: ParIter<Item = String>>(par: R, len: usize) {
    fn add(a: String, b: String) -> String {
        let a: i64 = a.parse().expect("ok");
        let b: i64 = b.parse().expect("ok");
        (a + b).to_string()
    }
    let expected = some_if(len > 0, (0..len).sum::<usize>().to_string());
    let result = par.reduce(add);
    assert_eq!(result, expected);
}

pub fn min<R: ParIter<Item = String>>(par: R, len: usize) {
    let expected = some_if(len > 0, 0.to_string());
    let result = par.min();
    assert_eq!(result, expected);
}

pub fn max<R: ParIter<Item = String>>(par: R, len: usize) {
    let expected = some_if(
        len > 0,
        match len {
            0 => 0.to_string(),
            x if x < 10 => (x - 1).to_string(),
            x if x < 100 => 9.to_string(),
            x if x < 1_000 => 99.to_string(),
            x if x < 1_0000 => 999.to_string(),
            x if x < 100_000 => 9_999.to_string(),
            x if x < 1_000_000 => 99_999.to_string(),
            x if x < 10_000_000 => 999_999.to_string(),
            _ => panic!("unhandled test case"),
        },
    );
    let result = par.max();
    assert_eq!(result, expected);
}

pub fn min_by<R: ParIter<Item = String>>(par: R, len: usize) {
    #[allow(clippy::ptr_arg)]
    fn compare(a: &String, b: &String) -> Ordering {
        let a: i64 = a.parse().expect("ok");
        let b: i64 = b.parse().expect("ok");
        match b % 2 {
            0 => a.cmp(&b),
            _ => Ordering::Less,
        }
    }
    let expected = (0..len).map(|x| x.to_string()).min_by(compare);
    let result = par.min_by(compare);
    assert_eq!(result, expected);
}

pub fn max_by<R: ParIter<Item = String>>(par: R, len: usize) {
    #[allow(clippy::ptr_arg)]
    fn compare(a: &String, b: &String) -> Ordering {
        let a: i64 = a.parse().expect("ok");
        let b: i64 = b.parse().expect("ok");
        match b % 2 {
            0 => a.cmp(&b),
            _ => Ordering::Greater,
        }
    }
    let expected = (0..len).map(|x| x.to_string()).max_by(compare);
    let result = par.max_by(compare);

    assert_eq!(result, expected);
}

pub fn min_by_key<R: ParIter<Item = String>>(par: R, len: usize) {
    #[allow(clippy::ptr_arg)]
    fn get_key(a: &String) -> usize {
        let a: i64 = a.parse().expect("ok");
        match a % 2 {
            0 => usize::MAX,
            _ => a as usize,
        }
    }
    let expected = (0..len).map(|x| x.to_string()).min_by_key(get_key);
    let result = par.min_by_key(get_key);
    assert_eq!(result, expected);
}

pub fn max_by_key<R: ParIter<Item = String>>(par: R, len: usize) {
    #[allow(clippy::ptr_arg)]
    fn get_key(a: &String) -> usize {
        let a: i64 = a.parse().expect("ok");
        match a % 2 {
            0 => 0,
            _ => a as usize,
        }
    }
    let expected = (0..len).map(|x| x.to_string()).max_by_key(get_key);
    let result = par.max_by_key(get_key);
    assert_eq!(result, expected);
}
