use crate::*;
use alloc::format;
use alloc::vec::Vec;
use std::string::{String, ToString};

fn par(n: usize) -> impl ParIter<Item = String> {
    (0..n).par().map(|x| x.to_string())
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

// fn to_option(par: impl ParIter<Item = Option<String>>) -> impl ParOptIter<Item = String> {}

#[test]
fn use_traits_collect() {
    let par = par(42);
    let par = map(par);
    let result = collect(par);
    assert_eq!(result.len(), 42);
}

#[test]
fn use_traits_reduce() {
    let par = par(42);
    let par = map(par);
    let result = count(par);
    assert_eq!(result, 42);
}

#[test]
fn use_traits_first() {
    let par = par(42);
    let par = map(par);
    let result = find(par);
    assert!(result.is_some());
}

#[test]
fn use_traits_into_option() {
    let par = par(42).map(Some);
    // let par = par.o
}
