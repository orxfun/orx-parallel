use crate::{
    collect_into::ParCollectInto, into_par::IntoPar, par_iterators::ParIter, IteratorIntoPar,
};
use orx_fixed_vec::FixedVec;
use orx_iterable::Collection;
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

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn expected(
    with_offset: bool,
    input: &impl Collection<Item = String>,
    map: impl Fn(String) -> String,
) -> Vec<String> {
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
    [Vec::<String>::new()],
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0), Vec::<String>::from_iter(offset()), SplitVec::<String>::from_iter(offset()), FixedVec::<String>::from_iter(offset()) ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect_into<I, C>(_: I, output: C, n: usize, nt: usize, chunk: usize)
where
    I: FromIterator<String> + Collection<Item = String> + IntoPar<ParItem = String>,
    C: ParCollectInto<String>,
{
    let vec = input::<Vec<_>>(n);
    let expected = expected(!output.is_empty(), &vec, |x| x);
    let input = vec.into_iter().filter(|x| x.as_str() != "?");
    let par = input.iter_into_par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [Vec::<String>::new()],
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect<I, C>(_: I, _: C, n: usize, nt: usize, chunk: usize)
where
    I: FromIterator<String> + Collection<Item = String> + IntoPar<ParItem = String>,
    C: ParCollectInto<String>,
{
    let vec = input::<Vec<_>>(n);
    let expected = expected(false, &vec, |x| x);
    let input = vec.into_iter().filter(|x| x.as_str() != "?");
    let par = input.iter_into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.collect();
    assert!(output.is_equal_to(&expected));
}

// collect - map

#[test_matrix(
    [Vec::<String>::new()],
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0), Vec::<String>::from_iter(offset()), SplitVec::<String>::from_iter(offset()), FixedVec::<String>::from_iter(offset()) ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect_into<I, C>(_: I, output: C, n: usize, nt: usize, chunk: usize)
where
    I: FromIterator<String> + Collection<Item = String> + IntoPar<ParItem = String>,
    C: ParCollectInto<String>,
{
    let map = |x| format!("{}!", x);
    let vec = input::<Vec<_>>(n);
    let expected = expected(!output.is_empty(), &vec, map);
    let input = vec.into_iter().filter(|x| x.as_str() != "?");
    let par = input.iter_into_par().num_threads(nt).chunk_size(chunk);
    let output = par.map(map).collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [Vec::<String>::new()],
    [Vec::<String>::new(), SplitVec::<String>::new(), FixedVec::<String>::new(0)],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn map_collect<I, C>(_: I, _: C, n: usize, nt: usize, chunk: usize)
where
    I: FromIterator<String> + Collection<Item = String> + IntoPar<ParItem = String>,
    C: ParCollectInto<String>,
{
    let map = |x| format!("{}!", x);
    let vec = input::<Vec<_>>(n);
    let expected = expected(false, &vec, map);
    let input = vec.into_iter().filter(|x| x.as_str() != "?");
    let par = input.iter_into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.map(map).collect();
    assert!(output.is_equal_to(&expected));
}
