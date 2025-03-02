use crate::{collect_into::ParCollectIntoCore, par_iterators::ParIter, Parallelizable};
use orx_fixed_vec::FixedVec;
use orx_iterable::{Collection, IntoCloningIterable};
use orx_split_vec::{Doubling, Linear, PseudoDefault, SplitVec};
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn expected(input: &impl Collection<Item = String>, map: impl Fn(String) -> String) -> Vec<String> {
    input.iter().cloned().map(map).collect()
}

// collect - empty

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect_into(n: usize, nt: usize, chunk: usize) {
    let vec = input::<Vec<_>>(n);
    let expected = expected(&vec, |x| x);
    let input = vec.iter().filter(|x| x.as_str() != "?").into_iterable();

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(Vec::new());
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(FixedVec::pseudo_default());
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(SplitVec::<_, Doubling>::pseudo_default());
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(SplitVec::<_, Linear>::pseudo_default());
    assert!(output.is_equal_to_ref(&expected));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect(n: usize, nt: usize, chunk: usize) {
    let vec = input::<Vec<_>>(n);
    let expected = expected(&vec, |x| x);
    let input = vec.iter().filter(|x| x.as_str() != "?").into_iterable();

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: Vec<&String> = par.collect();
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: FixedVec<&String> = par.collect();
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: SplitVec<&String, Doubling> = par.collect();
    assert!(output.is_equal_to_ref(&expected));

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: SplitVec<&String, Linear> = par.collect();
    assert!(output.is_equal_to_ref(&expected));
}

// collect - map

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect_into(n: usize, nt: usize, chunk: usize) {
    let map = |x: &String| format!("{}!", x);
    let map2 = |x: String| format!("{}!", x);

    let vec = input::<Vec<_>>(n);
    let expected = expected(&vec, map2);
    let input = vec.iter().filter(|x| x.as_str() != "?").into_iterable();

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: Vec<String> = par.map(map).collect_into(vec![]);
    assert!(output.is_equal_to(expected.as_slice()));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect(n: usize, nt: usize, chunk: usize) {
    let map = |x: &String| format!("{}!", x);
    let map2 = |x: String| format!("{}!", x);

    let vec = input::<Vec<_>>(n);
    let expected = expected(&vec, map2);
    let input = vec.iter().filter(|x| x.as_str() != "?").into_iterable();

    let par = input.par().num_threads(nt).chunk_size(chunk);
    let output: Vec<String> = par.map(map).collect();
    assert!(output.is_equal_to(expected.as_slice()));
}
