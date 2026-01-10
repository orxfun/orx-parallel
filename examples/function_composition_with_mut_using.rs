use clap::Parser;
use core::cell::UnsafeCell;

/*
This example demonstrates potential issues found by miri with different versions of actually dong exactly
the same thing:

* We compose functions which use a mutable reference to a variable.
* These functions access the value sequentially. We do not have a race condition.
* However, due to complexity of function composition with mutable lifetimes, we are unable to represent
  the correctness of the composition with mutable references.
* Miri remarks the problem since multiple closures hold a mutable reference to the same value. This is
  correct and a potential race-condition risk in multi-threaded scenarios.
* On the other hand, using transformation guarantees to send one clone of the used value to each one of
  the thread which is then used sequentially.

This example demonstrates and tests different approaches which would avoid having multiple mutable
references and fix the Miri error.

The versions implemented and tested are as follows:

cargo +nightly miri run --example function_composition_with_mut_using -- --version mut-ref-unsafe-lt1
> fails

cargo +nightly miri run --example function_composition_with_mut_using -- --version mut-ref-unsafe-lt2
> fails

cargo +nightly miri run --example function_composition_with_mut_using -- --version unsafe-cell-on-reduce
> fails

cargo +nightly miri run --example function_composition_with_mut_using -- --version unsafe-cell-on-all
> passes

cargo +nightly miri run --example function_composition_with_mut_using -- --version clone
> passes

cargo +nightly miri run --example function_composition_with_mut_using -- --version raw-ptr-all
> passes

*/

// # FAIL - mut ref & unsafe - 1

fn compose_mut_ref_unsafe_1<T, U, X, I, R>(
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

fn test_mut_ref_unsafe_1() {
    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_mut_ref_unsafe_1(xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(26));
}

// # FAIL - mut ref & unsafe - 2

fn compose_mut_ref_unsafe_2<T, U, M, X, I, R>(
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

fn test_mut_ref_unsafe_2() {
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

    let composed = compose_mut_ref_unsafe_2(map1, xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # FAIL - UnsafeCell reduce

fn compose_unsafe_cell_on_reduce<T, U, M, X, I, R>(
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

fn test_unsafe_cell_on_reduce() {
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

    let composed = compose_unsafe_cell_on_reduce(map1, xap1, reduce1);

    let s = UnsafeCell::new(String::new());
    let result = composed(&s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # PASS - UnsafeCell everything

fn compose_unsafe_cell_on_all<T, U, M, X, I, R>(
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

fn test_unsafe_cell_on_all() {
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

    let composed = compose_unsafe_cell_on_all(map1, xap1, reduce1);

    let s = UnsafeCell::new(String::new());
    let result = composed(&s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

// # PASS - clone

fn compose_clone<T, U, X, I, R>(xap1: X, reduce1: R) -> impl FnOnce(&mut U, T) -> Option<T>
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

fn test_clone() {
    let xap1 = |u: &mut String, x: i32| {
        u.push_str("1");
        [x, x + 2]
    };

    let reduce1 = |u: &mut String, x: i32, y: i32| {
        u.push_str("2");
        x + y
    };

    let composed = compose_clone(xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(26));
}

// # PASS - raw-ptr

fn compose_raw_ptr_all<T, U, M, X, I, R>(
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

fn test_raw_ptr_all() {
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

    let composed = compose_raw_ptr_all(map1, xap1, reduce1);

    let mut s = String::new();
    let result = composed(&mut s as *mut String, 12);
    dbg!(result, &s);

    assert_eq!(result, Some(28));
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum Version {
    MutRefUnsafeLt1,
    MutRefUnsafeLt2,
    UnsafeCellOnReduce,
    UnsafeCellOnAll,
    Clone,
    RawPtrAll,
}

#[derive(Parser, Debug)]
struct Args {
    /// Version of function composition to run and test with miri.
    #[arg(long, value_enum)]
    version: Version,
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");

    match args.version {
        Version::MutRefUnsafeLt1 => test_mut_ref_unsafe_1(),
        Version::MutRefUnsafeLt2 => test_mut_ref_unsafe_2(),
        Version::UnsafeCellOnReduce => test_unsafe_cell_on_reduce(),
        Version::UnsafeCellOnAll => test_unsafe_cell_on_all(),
        Version::Clone => test_clone(),
        Version::RawPtrAll => test_raw_ptr_all(),
    }
}

// mut-ref-unsafe-lt1, mut-ref-unsafe-lt2, unsafe-cell-on-reduce, unsafe-cell-on-all, clone, raw-ptr-all
