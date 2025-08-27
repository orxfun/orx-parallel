use crate::fibonacci;
use criterion::black_box;
use orx_parallel::*;
use test_case::test_case;

#[test_case(512, 4, 0, 3, "22", 220)]
#[test_case(1024, 3, 0, 3, "84", 840)]
#[test_case(1024, 4, 1, 3, "84", 840)]
#[test_case(1024, 2, 0, 2, "5", 50)]
fn par(n: usize, nt: usize, c: usize, until_num_digits: usize, until_digits: &str, min_len: usize) {
    let whilst = |x: &String| x.len() != until_num_digits || !x.starts_with(&until_digits);

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .iteration_order(IterationOrder::Arbitrary)
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            whilst(&x)
        })
        .collect();

    assert!(output.len() >= min_len);

    assert!(output.iter().all(whilst));
}

#[test_case(512, 4, 0, 3, "32", 320)]
#[test_case(1024, 3, 0, 3, "84", 840)]
#[test_case(1024, 4, 1, 3, "84", 840)]
#[test_case(1024, 2, 0, 2, "8", 80)]
fn map(n: usize, nt: usize, c: usize, until_num_digits: usize, until_digits: &str, min_len: usize) {
    let whilst = |x: &String| x.len() != until_num_digits || !x.starts_with(&until_digits);

    let input = 0..n;
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .iteration_order(IterationOrder::Arbitrary)
        .map(|x| x.to_string())
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            whilst(&x)
        })
        .collect();

    assert!(output.len() >= min_len);

    assert!(output.iter().all(whilst));
}

#[test_case(512, 0, 0, &["55"], &["55"], 511)]
#[test_case(512, 4, 1, &["55", "222"], &["55"], 221)]
#[test_case(512, 3, 0, &["55", "222"], &["55"], 221)]
#[test_case(512, 5, 0, &["333"], &["444"], 333)]
fn xap_filter(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[&str],
    filter_out: &[&str],
    min_len: usize,
) {
    let whilst = |x: &String| !stop_at.contains(&x.as_str());

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .iteration_order(IterationOrder::Arbitrary)
        .filter(|x| !filter_out.contains(&x.as_str()))
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            whilst(&x)
        })
        .collect();

    assert!(output.len() >= min_len);

    assert!(output.iter().all(whilst));
}

#[test_case(512, 0, 0, &["55"], &["55"], 511)]
#[test_case(512, 4, 1, &["55", "222"], &["55"], 221)]
#[test_case(512, 3, 0, &["55", "222"], &["55"], 221)]
#[test_case(512, 5, 0, &["333"], &["444"], 333)]
fn xap_filter_map(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[&str],
    filter_out: &[&str],
    min_len: usize,
) {
    let whilst = |x: &String| !stop_at.contains(&x.as_str());

    let input: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .iteration_order(IterationOrder::Arbitrary)
        .filter_map(|x| (!filter_out.contains(&x.as_str())).then_some(x))
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            whilst(&x)
        })
        .collect();

    assert!(output.len() >= min_len);

    assert!(output.iter().all(whilst));
}

#[test_case(1024, 0, 0, &[555], None, 555*3)]
#[test_case(1024, 4, 0, &[555], Some("!"), 555*3+1)]
#[test_case(1024, 3, 0, &[555], Some("?"), 555*3+2)]
#[test_case(1024, 3, 0, &[554,555,556,557,558,559], Some("?"), 554*3)]
fn xap_flat_map(
    n: usize,
    nt: usize,
    c: usize,
    stop_at: &[usize],
    stop_at_char: Option<&str>,
    min_len: usize,
) {
    let whilst = |x: &String| {
        let s = match x.ends_with("!") || x.ends_with("?") {
            true => &x[0..(x.len() - 1)],
            false => x.as_str(),
        };
        let num: usize = s.parse().unwrap();
        !stop_at.contains(&num)
            || stop_at_char
                .map(|stop_at_char| !x.ends_with(stop_at_char))
                .unwrap_or(false)
    };

    let input = 0..n;
    let output: Vec<_> = input
        .into_par()
        .num_threads(nt)
        .chunk_size(c)
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(|i| [i.to_string(), format!("{i}!"), format!("{i}?")])
        .take_while(|x| {
            let _fib = black_box(fibonacci(42));
            whilst(&x)
        })
        .collect();

    assert!(output.len() >= min_len);

    assert!(output.iter().all(whilst));
}
