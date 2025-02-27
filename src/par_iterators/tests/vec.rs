use crate::{
    collect_into::ParCollectInto, into_par::IntoPar, par_iterators::ParIter,
    parallelizable::Parallelizable, parallelizable_collection::ParallelizableCollection,
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

fn expected(with_offset: bool, input: &[String], map: impl Fn(String) -> String) -> Vec<String> {
    match with_offset {
        true => {
            let mut vec = offset();
            vec.extend(input.iter().cloned().map(map));
            vec
        }
        false => input.iter().cloned().map(map).collect(),
    }
}

// collect - empty

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0), Vec::<String>::from_iter(offset()), SplitVec::<String>::from_iter(offset()), FixedVec::<String>::from_iter(offset()) ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect_into<C: ParCollectInto<String>>(output: C, n: usize, nt: usize, chunk: usize) {
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(!output.is_empty(), &input, |x| x);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect<C: ParCollectInto<String>>(_: C, n: usize, nt: usize, chunk: usize) {
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(false, &input, |x| x);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.collect();
    assert!(output.is_equal_to(&expected));
}

// collect - map

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0), Vec::<String>::from_iter(offset()), SplitVec::<String>::from_iter(offset()), FixedVec::<String>::from_iter(offset()) ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect_into<C: ParCollectInto<String>>(output: C, n: usize, nt: usize, chunk: usize) {
    let map = |x| format!("{}!", x);
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(!output.is_empty(), &input, map);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output = par.map(map).collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect<C: ParCollectInto<String>>(_: C, n: usize, nt: usize, chunk: usize) {
    let map = |x| format!("{}!", x);
    let input = input(n, |x| (x + 10).to_string());
    let expected = expected(false, &input, map);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.map(map).collect();
    assert!(output.is_equal_to(&expected));
}

// into - as

#[test]
fn parallelizable() {
    fn take<'a>(a: impl Parallelizable<ParItem = &'a String>) {
        let _ = a.par();
    }
    let input = input(7, |x| (x + 10).to_string());
    take(&input);
}

#[test]
fn parallelizable_collection() {
    fn take(a: &impl ParallelizableCollection<ParItem = String>) {
        let _ = a.par();
    }
    let input = input(7, |x| (x + 10).to_string());
    take(&input);
}
