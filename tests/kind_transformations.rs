use orx_parallel::*;
use std::string::{String, ToString};

fn par(n: usize) -> impl ParIter<Item = String> + ParIterEnumarable {
    (0..n).par().map(|x| x.to_string())
}

fn par_use(n: usize) -> impl ParUseIter<U = String, Item = String> + ParUseIterEnumarable {
    (0..n)
        .par()
        .using_clone("abc".to_string())
        .map(|_, x| x.to_string())
}

fn map(par: impl ParIter<Item = String>) -> impl ParIter<Item = String> {
    par.map(|x| format!("{x}!"))
        .num_threads(2)
        .chunk_size(0)
        .iteration_order(IterationOrder::Ordered)
}

fn collect(par: impl ParIter<Item = String>) -> Vec<String> {
    par.num_threads(3).chunk_size(1).collect()
}

fn count(par: impl ParIter<Item = String>) -> usize {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_| 1)
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

fn find(par: impl ParIter<Item = String>) -> Option<String> {
    par.filter(|x| x.len() > 2)
        .num_threads(6)
        .chunk_size(3)
        .first()
}

fn count_opt(par: impl ParOptIter) -> Option<usize> {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_| 1)
        .reduce(|a, b| a + b)
        .map(|x| x.unwrap_or(0))
}

fn count_res(par: impl ParResIter<Error = char>) -> Result<usize, char> {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_| 1)
        .reduce(|a, b| a + b)
        .map(|x| x.unwrap_or(0))
}

fn count_use(par: impl ParUseIter<U = String>) -> usize {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_use, _| 1)
        .reduce(|_use, a, b| a + b)
        .unwrap_or(0)
}

fn count_use_opt(par: impl ParUseOptIter<U = String>) -> Option<usize> {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_use, _| 1)
        .reduce(|_use, a, b| a + b)
        .map(|x| x.unwrap_or(0))
}

fn count_use_res(par: impl ParUseResIter<U = String, Error = char>) -> Result<usize, char> {
    par.num_threads(1)
        .chunk_size(7)
        .map(|_use, _| 1)
        .reduce(|_use, a, b| a + b)
        .map(|x| x.unwrap_or(0))
}

#[test]
fn kind_collect() {
    let par = par(42);
    let par = map(par);
    let result = collect(par);
    assert_eq!(result.len(), 42);
}

#[test]
fn kind_reduce() {
    let par = par(42);
    let par = map(par);
    let result = count(par);
    assert_eq!(result, 42);
}

#[test]
fn kind_first() {
    let par = par(42);
    let par = map(par);
    let result = find(par);
    assert!(result.is_some());
}

#[test]
fn kind_into_optional() {
    let par = par(42).map(Some);
    let par = par.into_optional();
    let result = count_opt(par);
    assert_eq!(result, Some(42));
}

#[test]
fn kind_into_optional_use() {
    {
        let par = par(42).map(Some);
        let par = par.into_optional();

        let u = |x: usize| x.to_string();
        let par = par.using(u);

        let result = count_use_opt(par);
        assert_eq!(result, Some(42));
    }

    {
        let par = par(42).map(Some);
        let par = par.into_optional();

        let u = String::from("42");
        let par = par.using_clone(u);

        let result = count_use_opt(par);
        assert_eq!(result, Some(42));
    }
}

#[test]
fn kind_into_use_optional() {
    {
        let u = |x: usize| x.to_string();
        let par = par(42);
        let par = par.using(u);
        let par = par.map(|_use, x| Some(x));
        let par = par.into_optional();
        let result = count_use_opt(par);
        assert_eq!(result, Some(42));
    }

    {
        let u = String::from("42");
        let par = par(42);
        let par = par.using_clone(u);
        let par = par.map(|_use, x| Some(x));
        let par = par.into_optional();
        let result = count_use_opt(par);
        assert_eq!(result, Some(42));
    }
}

#[test]
fn kind_into_fallible() {
    let par = par(42).map(|x| Result::<_, char>::Ok(x));
    let par = par.into_fallible();
    let result = count_res(par);
    assert_eq!(result, Ok(42));
}

#[test]
fn kind_into_fallible_use() {
    {
        let par = par(42).map(|x| Result::<_, char>::Ok(x));
        let par = par.into_fallible();

        let u = |x: usize| x.to_string();
        let par = par.using(u);

        let result = count_use_res(par);
        assert_eq!(result, Ok(42));
    }

    {
        let par = par(42).map(|x| Result::<_, char>::Ok(x));
        let par = par.into_fallible();

        let u = String::from("42");
        let par = par.using_clone(u);

        let result = count_use_res(par);
        assert_eq!(result, Ok(42));
    }
}

#[test]
fn kind_into_use_fallible() {
    {
        let u = |x: usize| x.to_string();
        let par = par(42);
        let par = par.using(u);

        let par = par.map(|_use, x| Result::<_, char>::Ok(x));
        let par = par.into_fallible();

        let result = count_use_res(par);
        assert_eq!(result, Ok(42));
    }

    {
        let u = String::from("42");
        let par = par(42);
        let par = par.using_clone(u);

        let par = par.map(|_use, x| Result::<_, char>::Ok(x));
        let par = par.into_fallible();

        let result = count_use_res(par);
        assert_eq!(result, Ok(42));
    }
}

#[test]
fn kind_use() {
    {
        let u = |x: usize| x.to_string();
        let par = par(42);
        let par = par.using(u);
        let result = count_use(par);
        assert_eq!(result, 42);
    }

    {
        let u = String::from("42");
        let par = par(42);
        let par = par.using_clone(u);
        let result = count_use(par);
        assert_eq!(result, 42);
    }
}

// enumerate

#[test]
fn kind_enumerate() {
    let par = par(42);
    let par = par.enumerate();
    let par = par.map(|(_i, x)| x);
    let result = count(par);
    assert_eq!(result, 42);
}

#[test]
fn kind_use_enumerate() {
    let par = par_use(42);
    let par = par.enumerate();
    let par = par.map(|u, (_i, x): (usize, String)| {
        *u = format!("{u}!");
        x
    });
    let result = count_use(par);
    assert_eq!(result, 42);
}
