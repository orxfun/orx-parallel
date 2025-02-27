use crate::{
    collect_into::{ParCollectInto, ParCollectIntoCore},
    into_par::IntoPar,
    par_iterators::ParIter,
};
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

const N_OFFSET: usize = 13;

fn offset() -> Vec<String> {
    vec!["x".to_string(); N_OFFSET]
}

fn input(n: usize, elem: impl Fn(usize) -> String) -> Vec<String> {
    let mut vec = Vec::with_capacity(n + 17);
    for i in 0..n {
        vec.push(elem(i));
    }
    vec
}

fn expected(with_offset: bool, input: &[String]) -> Vec<String> {
    match with_offset {
        true => {
            let mut vec = offset();
            vec.extend(input.iter().cloned());
            vec
        }
        false => input.iter().cloned().collect(),
    }
}

#[test_matrix(
    [
        Vec::<String>::new(),
        SplitVec::<String>::new(),
        FixedVec::<String>::new(0),
        Vec::<String>::from_iter(offset()),
        SplitVec::<String>::from_iter(offset()),
        FixedVec::<String>::from_iter(offset())
    ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect_into<C: ParCollectInto<String>>(output: C, n: usize, nt: usize, chunk: usize) {
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(!output.is_empty(), &input);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [
        Vec::<String>::new(),
        SplitVec::<String>::new(),
        FixedVec::<String>::new(0),
        Vec::<String>::from_iter(offset()),
        SplitVec::<String>::from_iter(offset()),
        FixedVec::<String>::from_iter(offset())
    ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect<C: ParCollectInto<String>>(_: C, n: usize, nt: usize, chunk: usize) {
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(false, &input);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.collect();
    assert!(output.is_equal_to(&expected));
}
