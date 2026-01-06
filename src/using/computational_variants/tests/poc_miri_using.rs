use core::cell::UnsafeCell;
use std::{dbg, string::String};

// # FAIL - raw pointer & unsafe - 1

fn compose_miri_failing_ref1<T, U, X, I, R>(
    xap1: X,
    reduce1: R,
) -> impl FnOnce(&mut U, T) -> Option<T>
where
    X: Fn(&mut U, T) -> I + Clone,
    R: Fn(&mut U, T, T) -> T + Clone,
    I: IntoIterator<Item = T> + Default,
{
    move |u: &mut U, x: T| {
        let xap1 = xap1.clone();
        let u2 = unsafe {
            &mut *{
                let p: *mut U = u;
                p
            }
        };
        let first = xap1(u, x);
        first.into_iter().reduce(|x, y| reduce1(u2, x, y))
    }
}

#[test]
fn u_miri_failing_ref1() {
    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_failing_ref1(xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(26));
}

// # FAIL - raw pointer & unsafe - 2

fn compose_miri_failing_ref2<T, U, M, X, I, R>(
    map1: M,
    xap1: X,
    reduce1: R,
) -> impl FnOnce(&mut U, T) -> Option<T>
where
    M: Fn(&mut U, T) -> T + Clone,
    X: Fn(&mut U, T) -> I + Clone,
    I: IntoIterator<Item = T> + Default,
    R: Fn(&mut U, T, T) -> T + Clone,
    U: 'static,
{
    let xap_map = move |u: &mut U, x: T| {
        let u2 = unsafe {
            &mut *{
                let p: *mut U = u;
                p
            }
        };
        let first = xap1(u, x);
        first.into_iter().map(move |x| map1(u2, x))
    };

    let composed = move |u: &mut U, x: T| {
        let values = xap_map(u, x);
        values.reduce(|x, y| reduce1(u, x, y))
    };

    composed
}

#[test]
fn u_miri_failing_ref2() {
    let map1 = |u: &mut String, x: i32| {
        u.push_str("0");
        x + 1
    };

    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_failing_ref2(map1, xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # FAIL - UnsafeCell reduce

fn compose_miri_failing_unsafe_cell1<T, U, M, X, I, R>(
    map1: M,
    xap1: X,
    reduce1: R,
) -> impl FnOnce(&UnsafeCell<U>, T) -> Option<T>
where
    M: Fn(&mut U, T) -> T + Clone,
    X: Fn(&mut U, T) -> I + Clone,
    I: IntoIterator<Item = T> + Default,
    R: Fn(&mut U, T, T) -> T + Clone,
    U: 'static,
{
    let xap_map = move |u: &mut U, x: T| {
        let u2 = unsafe {
            &mut *{
                let p: *mut U = u;
                p
            }
        };
        let first = xap1(u, x);
        first.into_iter().map(move |x| map1(u2, x))
    };

    let composed = move |u: &UnsafeCell<U>, x: T| {
        let values = xap_map(unsafe { &mut *u.get() }, x);
        values.reduce(|x, y| reduce1(unsafe { &mut *u.get() }, x, y))
    };

    composed
}

#[test]
fn u_miri_failing_unsafe_cell1() {
    let map1 = |u: &mut String, x: i32| {
        u.push_str("0");
        x + 1
    };

    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_failing_unsafe_cell1(map1, xap1, reduce1);

    let s = UnsafeCell::new(String::new());
    let result = composed(&s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # PASS - UnsafeCell everything

fn compose_miri_passing_unsafe_cell2<T, U, M, X, I, R>(
    map1: M,
    xap1: X,
    reduce1: R,
) -> impl FnOnce(&UnsafeCell<U>, T) -> Option<T>
where
    M: Fn(&mut U, T) -> T + Clone,
    X: Fn(&mut U, T) -> I + Clone,
    I: IntoIterator<Item = T> + Default,
    R: Fn(&mut U, T, T) -> T + Clone,
    U: 'static,
{
    let map2 = move |u: &UnsafeCell<U>, x: T| map1(unsafe { &mut *u.get() }, x);

    let xap_map = move |u: &UnsafeCell<U>, x: T| {
        let u2 = unsafe {
            &*{
                let p: *const UnsafeCell<_> = u;
                p
            }
        };
        let first = xap1(unsafe { &mut *u.get() }, x);
        first.into_iter().map(move |x| map2(u2, x))
    };

    let composed = move |u: &UnsafeCell<U>, x: T| {
        let values = xap_map(u, x);
        values.reduce(|x, y| reduce1(unsafe { &mut *u.get() }, x, y))
    };

    composed
}

#[test]
fn u_miri_passing_unsafe_cell2() {
    let map1 = |u: &mut String, x: i32| {
        u.push_str("0");
        x + 1
    };

    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_passing_unsafe_cell2(map1, xap1, reduce1);

    let s = UnsafeCell::new(String::new());
    let result = composed(&s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # PASS - UnsafeCell

fn compose_miri_passing_unsafe_cell<T, U, X, I, R>(
    xap1: X,
    reduce1: R,
) -> impl FnOnce(&UnsafeCell<U>, T) -> Option<T>
where
    X: Fn(&mut U, T) -> I + Clone,
    R: Fn(&mut U, T, T) -> T + Clone,
    I: IntoIterator<Item = T> + Default,
{
    move |u: &UnsafeCell<U>, x: T| {
        let xap1 = xap1.clone();
        let first = xap1(unsafe { &mut *u.get() }, x);
        first
            .into_iter()
            .reduce(|x, y| reduce1(unsafe { &mut *u.get() }, x, y))
    }
}

#[test]
fn u_miri_passing_unsafe_cell() {
    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_passing_unsafe_cell(xap1, reduce1);

    let s = UnsafeCell::new(String::new());
    let result = composed(&s, 12);
    let s = s.into_inner();
    dbg!(result, &s);

    assert_eq!(result, Some(26));
}

// # PASS - clone

fn compose_miri_passing<T, U, X, I, R>(xap1: X, reduce1: R) -> impl FnOnce(&mut U, T) -> Option<T>
where
    U: Clone,
    X: Fn(&mut U, T) -> I + Clone,
    R: Fn(&mut U, T, T) -> T + Clone,
    I: IntoIterator<Item = T> + Default,
{
    move |u: &mut U, x: T| {
        let xap1 = xap1.clone();
        let u2 = &mut u.clone();
        let first = xap1(u, x);
        first.into_iter().reduce(|x, y| reduce1(u2, x, y))
    }
}

#[test]
fn u_miri_passing() {
    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_passing(xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(26));
}

// # PASS - raw-ptr

fn compose_miri_passing_raw_ptr<T, U, M, X, I, R>(
    map1: M,
    xap1: X,
    reduce1: R,
) -> impl FnOnce(*mut U, T) -> Option<T>
where
    M: Fn(&mut U, T) -> T + Clone,
    X: Fn(&mut U, T) -> I + Clone,
    I: IntoIterator<Item = T> + Default,
    R: Fn(&mut U, T, T) -> T + Clone,
    U: 'static,
{
    let map2 = move |u: *mut U, x: T| map1(unsafe { &mut *u }, x);

    let xap_map = move |u: *mut U, x: T| {
        let first = xap1(unsafe { &mut *u }, x);
        first.into_iter().map(move |x| map2(unsafe { &mut *u }, x))
    };

    let composed = move |u: *mut U, x: T| {
        let values = xap_map(u, x);
        values.reduce(|x, y| reduce1(unsafe { &mut *u }, x, y))
    };

    composed
}

#[test]
fn u_miri_passing_raw_ptr() {
    let map1 = |u: &mut String, x: i32| {
        u.push_str("0");
        x + 1
    };

    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_miri_passing_raw_ptr(map1, xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s as *mut String, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}
