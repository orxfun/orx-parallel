use crate::{collect_into::ParCollectInto, into_par::IntoPar, par_iterators::ParIter};
use orx_fixed_vec::FixedVec;
use orx_iterable::{Collection, Iterable};
use orx_split_vec::SplitVec;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

const N_OFFSET: usize = 13;

fn offset() -> Vec<usize> {
    vec![9; N_OFFSET]
}

fn input<O: FromIterator<usize>>(n: usize) -> O {
    (0..n).collect()
}

fn expected(
    with_offset: bool,
    input: impl Iterable<Item = usize>,
    map: impl Fn(usize) -> usize,
) -> Vec<usize> {
    match with_offset {
        true => {
            let mut vec = offset();
            vec.extend(input.iter().map(map));
            vec
        }
        false => input.iter().map(map).collect(),
    }
}

// collect - empty

#[test_matrix(
    [Vec::<usize>::new(), SplitVec::<usize>::new(), FixedVec::<usize>::new(0), Vec::<usize>::from_iter(offset()), SplitVec::<usize>::from_iter(offset()), FixedVec::<usize>::from_iter(offset()) ],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect_into<C>(output: C, n: usize, nt: usize, chunk: usize)
where
    C: ParCollectInto<usize>,
{
    let input = 0..n;
    let expected = expected(!output.is_empty(), input.clone(), |x| x);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output = par.collect_into(output);
    assert!(output.is_equal_to(&expected));
}

#[test_matrix(
    [Vec::<usize>::new(), SplitVec::<usize>::new(), FixedVec::<usize>::new(0)],
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn empty_collect<C>(_: C, n: usize, nt: usize, chunk: usize)
where
    C: ParCollectInto<usize>,
{
    let input = 0..n;
    let expected = expected(false, input.clone(), |x| x);
    let par = input.into_par().num_threads(nt).chunk_size(chunk);
    let output: C = par.collect();
    assert!(output.is_equal_to(&expected));
}
