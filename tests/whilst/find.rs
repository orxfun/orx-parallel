use crate::fibonacci;
use std::hint::black_box;
use orx_parallel::*;
use test_case::test_case;

#[test_case(511, 0, 0, &[333], &[333], None)]
#[test_case(511, 0, 0, &[333], &[332], Some(332))]
#[test_case(511, 0, 0, &[222, 333], &[223, 224, 225, 332], None)]
#[test_case(511, 0, 0, &[171], &[172], None)]
fn par(n: usize, nt: usize, c: usize, stop_at: &[usize], find: &[usize], expected: Option<usize>) {
    let filter = |x: &String| find.contains(&x.parse().unwrap());

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            let num: usize = x.parse().unwrap();
            !stop_at.contains(&num)
        })
        .find(filter)
        .map(|x| x.parse().unwrap());

    assert_eq!(output, expected);
}

#[test_case(511, 0, 0, &[333], &[334], Some(334))]
#[test_case(511, 0, 0, &[334], &[332], Some(332))]
#[test_case(511, 0, 0, &[222, 333], &[223, 224, 225, 332], None)]
#[test_case(511, 0, 0, &[170], &[172], None)]
fn filter(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[usize],
    find: &[usize],
    expected: Option<usize>,
) {
    let filter = |x: &String| find.contains(&x.parse().unwrap());

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .filter(|x| x.parse::<usize>().unwrap() % 2 == 0) // only evens remain
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            let num: usize = x.parse().unwrap();
            !stop_at.contains(&num)
        })
        .find(filter)
        .map(|x| x.parse().unwrap());

    assert_eq!(output, expected);
}

#[test_case(511, 0, 0, &["333!"], &[333], Some("333".to_string()))]
#[test_case(511, 0, 0, &["334?"], &[332], Some("332".to_string()))]
#[test_case(511, 0, 0, &["222!","333"], &[223, 224, 225, 332], None)]
#[test_case(511, 0, 0, &["170!"], &[170], Some("170".to_string()))]
fn flat_map(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[&str],
    find: &[usize],
    expected: Option<String>,
) {
    let filter = |x: &String| {
        let s = match x.ends_with("!") || x.ends_with("?") {
            true => &x[0..(x.len() - 1)],
            false => x.as_str(),
        };
        let num: usize = s.parse().unwrap();
        find.contains(&num)
    };

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .flat_map(|i| [i.to_string(), format!("{i}!"), format!("{i}?")])
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            !stop_at.contains(&x.as_str())
        })
        .find(filter)
        .map(|x| x.parse().unwrap());

    assert_eq!(output, expected);
}
