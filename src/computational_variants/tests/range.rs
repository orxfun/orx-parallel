use crate::{test_utils::*, *};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_iterable::Iterable;
use orx_split_vec::SplitVec;
use test_case::test_matrix;

const N_OFFSET: usize = 13;

fn offset() -> Vec<usize> {
    vec![9; N_OFFSET]
}

fn expected<T>(
    with_offset: bool,
    input: impl Iterable<Item = usize>,
    map: impl Fn(usize) -> T + Copy,
) -> Vec<T> {
    match with_offset {
        true => {
            let mut vec: Vec<_> = offset().into_iter().map(map).collect();
            vec.extend(input.iter().map(map));
            vec
        }
        false => input.iter().map(map).collect(),
    }
}

// collect - empty

#[test_matrix(
    [Vec::<usize>::new(), SplitVec::<usize>::new(), FixedVec::<usize>::new(0), Vec::<usize>::from_iter(offset()), SplitVec::<usize>::from_iter(offset()), FixedVec::<usize>::from_iter(offset()) ],
    N, NT, CHUNK)
]
fn empty_collect_into<C>(output: C, n: &[usize], nt: &[usize], chunk: &[usize])
where
    C: ParCollectInto<usize> + Clone,
{
    let test = |n, nt, chunk| {
        let input = 0..n;
        let expected = expected(!output.is_empty(), input.clone(), |x| x);
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.collect_into(output.clone());
        assert!(output.is_equal_to(&expected));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(
    [Vec::<usize>::new(), SplitVec::<usize>::new(), FixedVec::<usize>::new(0)],
    N, NT, CHUNK)
]
fn empty_collect<C>(_: C, n: &[usize], nt: &[usize], chunk: &[usize])
where
    C: ParCollectInto<usize>,
{
    let test = |n, nt, chunk| {
        let input = 0..n;
        let expected = expected(false, input.clone(), |x| x);
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: C = par.collect();
        assert!(output.is_equal_to(&expected));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

// collect - map

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    N, NT, CHUNK)
]
fn map_collect_into<C>(output: C, n: &[usize], nt: &[usize], chunk: &[usize])
where
    C: ParCollectInto<String> + Clone,
{
    let test = |n, nt, chunk| {
        let map = |x: usize| x.to_string();
        let input = 0..n;
        let expected = expected(!output.is_empty(), input.clone(), map);
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output = par.map(map).collect_into(output.clone());
        assert!(output.is_equal_to(&expected));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}

#[test_matrix(
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    N, NT, CHUNK)
]
fn map_collect<C>(_: C, n: &[usize], nt: &[usize], chunk: &[usize])
where
    C: ParCollectInto<String>,
{
    let test = |n, nt, chunk| {
        let map = |x: usize| x.to_string();
        let input = 0..n;
        let expected = expected(false, input.clone(), map);
        let par = input.into_par().num_threads(nt).chunk_size(chunk);
        let output: C = par.map(map).collect();
        assert!(output.is_equal_to(&expected));
    };
    test_n_nt_chunk(n, nt, chunk, test);
}
