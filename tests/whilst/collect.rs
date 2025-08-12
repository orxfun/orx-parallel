use crate::fibonacci;
use criterion::black_box;
use orx_parallel::*;
use test_case::test_case;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

#[test_case(0, 0, 0, 0, "0")]
#[test_case(512, 4, 0, 3, "22")]
#[test_case(1024, 3, 0, 3, "84")]
#[test_case(1024, 4, 1, 3, "84")]
#[test_case(1024, 2, 0, 2, "5")]
fn par(n: usize, nt: usize, c: usize, until_num_digits: usize, until_digits: &str) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            x.len() != until_num_digits || !x.starts_with(&until_digits)
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .map(|x| x.to_string())
        .take_while(|x| x.len() != until_num_digits || !x.starts_with(&until_digits))
        .collect();

    assert_eq!(output, expected);
}

#[test_case(0, 0, 0, 0, "0")]
#[test_case(512, 4, 0, 3, "82")]
#[test_case(1024, 3, 0, 3, "84")]
#[test_case(1024, 4, 1, 3, "84")]
#[test_case(1024, 2, 0, 2, "8")]
fn map(n: usize, nt: usize, c: usize, until_num_digits: usize, until_digits: &str) {
    let input = 0..n;
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .map(|x| x.to_string())
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            x.len() != until_num_digits || !x.starts_with(&until_digits)
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .map(|x| x.to_string())
        .take_while(|x| x.len() != until_num_digits || !x.starts_with(&until_digits))
        .collect();

    assert_eq!(output, expected);
}

#[test_case(512, 0, 0, &["55"], &["55"])]
#[test_case(512, 4, 1, &["55", "222"], &["55"])]
#[test_case(512, 3, 0, &["55", "222"], &["55"])]
#[test_case(512, 5, 0, &["333"], &["444"])]
fn xap_filter(n: usize, nt: usize, c: usize, stop_at: &[&str], filter_out: &[&str]) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .filter(|x| !filter_out.contains(&x.as_str()))
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            !stop_at.contains(&x.as_str())
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .map(|x| x.to_string())
        .filter(|x| !filter_out.contains(&x.as_str()))
        .take_while(|x| !stop_at.contains(&x.as_str()))
        .collect();

    assert_eq!(output, expected);
}

#[test_case(512, 0, 0, &["55"], &["55"])]
#[test_case(512, 4, 1, &["55", "222"], &["55"])]
#[test_case(512, 3, 0, &["55", "222"], &["55"])]
#[test_case(512, 5, 0, &["333"], &["444"])]
fn xap_filter_map(n: usize, nt: usize, c: usize, stop_at: &[&str], filter_out: &[&str]) {
    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .filter_map(|x| (!filter_out.contains(&x.as_str())).then_some(x))
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            !stop_at.contains(&x.as_str())
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .map(|x| x.to_string())
        .filter_map(|x| (!filter_out.contains(&x.as_str())).then_some(x))
        .take_while(|x| !stop_at.contains(&x.as_str()))
        .collect();

    assert_eq!(output, expected);
}

#[test_case(1024, 0, 0, &[555], None)]
#[test_case(1024, 4, 0, &[555], Some("!"))]
#[test_case(1024, 3, 0, &[555], Some("?"))]
#[test_case(1024, 3, 0, &[554,555,556,557,558,559], Some("?"))]
fn xap_flat_map(n: usize, nt: usize, c: usize, stop_at: &[usize], stop_at_char: Option<&str>) {
    let input = 0..n;
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .flat_map(|i| [i.to_string(), format!("{i}!"), format!("{i}?")])
        .whilst(|x| {
            let _fib = black_box(fibonacci(42));
            let s = match x.ends_with("!") || x.ends_with("?") {
                true => &x[0..(x.len() - 1)],
                false => x.as_str(),
            };
            let num: usize = s.parse().unwrap();
            !stop_at.contains(&num)
                || stop_at_char
                    .map(|stop_at_char| !x.ends_with(stop_at_char))
                    .unwrap_or(false)
        })
        .collect();
    let expected: Vec<_> = (0..n)
        .flat_map(|i| [i.to_string(), format!("{i}!"), format!("{i}?")])
        .take_while(|x| {
            let s = match x.ends_with("!") || x.ends_with("?") {
                true => &x[0..(x.len() - 1)],
                false => x.as_str(),
            };
            let num: usize = s.parse().unwrap();
            !stop_at.contains(&num)
                || stop_at_char
                    .map(|stop_at_char| !x.ends_with(stop_at_char))
                    .unwrap_or(false)
        })
        .collect();

    assert_eq!(output, expected);
}
