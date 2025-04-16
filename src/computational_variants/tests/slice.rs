use crate::{collect_into::ParCollectIntoCore, test_utils::test_n_nt_chunk, *};
use orx_fixed_vec::FixedVec;
use orx_iterable::Collection;
use orx_split_vec::{Doubling, Linear, PseudoDefault, SplitVec};
use test_case::test_matrix;

#[cfg(not(miri))]
const N: &[usize] = &[8025, 42735];
#[cfg(not(miri))]
const NT: &[usize] = &[1, 2, 4];
#[cfg(not(miri))]
const CHUNK: &[usize] = &[1, 64, 1024];

#[cfg(miri)]
const N: &[usize] = &[37, 125];
#[cfg(miri)]
const NT: &[usize] = &[3];
#[cfg(miri)]
const CHUNK: &[usize] = &[1, 64];

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn expected(input: &impl Collection<Item = String>, map: impl Fn(String) -> String) -> Vec<String> {
    input.iter().cloned().map(map).collect()
}

// collect - empty

#[test_matrix(N, NT, CHUNK)]
fn empty_collect_into(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let vec = input::<Vec<String>>(n);
        let input = vec.as_slice();
        let expected = expected(&vec, |x| x);

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.collect_into(Vec::new());
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.collect_into(FixedVec::pseudo_default());
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.collect_into(SplitVec::<_, Doubling>::pseudo_default());
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.collect_into(SplitVec::<_, Linear>::pseudo_default());
        assert!(output.is_equal_to_ref(expected.as_slice()));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn empty_collect(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let vec = input::<Vec<String>>(n);
        let input = vec.as_slice();
        let expected = expected(&vec, |x| x);

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<&String> = par.collect();
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: FixedVec<&String> = par.collect();
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: SplitVec<&String, Doubling> = par.collect();
        assert!(output.is_equal_to_ref(expected.as_slice()));

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: SplitVec<&String, Linear> = par.collect();
        assert!(output.is_equal_to_ref(expected.as_slice()));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

// collect - map

#[test_matrix(N, NT, CHUNK)]
fn map_collect_into(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let map = |x: &String| format!("{}!", x);
        let map2 = |x: String| format!("{}!", x);

        let vec = input::<Vec<String>>(n);
        let input = vec.as_slice();
        let expected = expected(&vec, map2);

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<String> = par.map(map).collect_into(vec![]);
        assert!(output.is_equal_to(expected.as_slice()));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(N, NT, CHUNK)]
fn map_collect(n: &[usize], nt: &[usize], chunk: &[usize]) {
    let test = |n, nt, chunk| {
        let map = |x: &String| format!("{}!", x);
        let map2 = |x: String| format!("{}!", x);

        let vec = input::<Vec<String>>(n);
        let input = vec.as_slice();
        let expected = expected(&vec, map2);

        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: Vec<String> = par.map(map).collect();
        assert!(output.is_equal_to(expected.as_slice()));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
