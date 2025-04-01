use crate::*;
use orx_concurrent_vec::ConcurrentVec;
use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [8025, 42735];

fn input<O: FromIterator<String>>(n: usize) -> O {
    let elem = |x: usize| (x + 10).to_string();
    (0..n).map(elem).collect()
}

fn sorted(mut x: Vec<String>) -> Vec<String> {
    x.sort();
    x
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn inspect_empty(n: usize, nt: usize, chunk: usize) {
    let input = || input::<Vec<_>>(n);

    let par = input().into_par().num_threads(nt).chunk_size(chunk);
    let vec = ConcurrentVec::new();
    par.inspect(|x| {
        _ = vec.push(x.clone());
    })
    .count();

    assert_eq!(sorted(vec.to_vec()), sorted(input()));
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn inspect_map(n: usize, nt: usize, chunk: usize) {
    let input = || input::<Vec<_>>(n);
    let map = |x| format!("{}!", x);

    let par = input().into_par().num_threads(nt).chunk_size(chunk);
    let par = par.map(&map);
    let vec = ConcurrentVec::new();
    par.inspect(|x| {
        _ = vec.push(x.clone());
    })
    .count();

    assert_eq!(
        sorted(vec.to_vec()),
        sorted(input().into_iter().map(map).collect())
    );
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn inspect_xap_flat_map(n: usize, nt: usize, chunk: usize) {
    let input = || input::<Vec<_>>(n);
    let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();

    let par = input().into_par().num_threads(nt).chunk_size(chunk);
    let par = par.flat_map(flat_map);
    let vec = ConcurrentVec::new();
    par.inspect(|x| {
        _ = vec.push(x.clone());
    })
    .count();

    assert_eq!(
        sorted(vec.to_vec()),
        sorted(input().into_iter().flat_map(flat_map).collect())
    );
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn inspect_xap_filter_map(n: usize, nt: usize, chunk: usize) {
    let input = || input::<Vec<_>>(n);
    let filter_map = |x: String| x.starts_with('3').then_some(x);

    let par = input().into_par().num_threads(nt).chunk_size(chunk);
    let par = par.filter_map(filter_map);
    let vec = ConcurrentVec::new();
    par.inspect(|x| {
        _ = vec.push(x.clone());
    })
    .count();

    assert_eq!(
        sorted(vec.to_vec()),
        sorted(input().into_iter().filter_map(filter_map).collect())
    );
}

#[test_matrix(
    [0, 1, N[0], N[1]],
    [1, 2, 4],
    [1, 64, 1024])
]
fn inspect_xap_filter_xap(n: usize, nt: usize, chunk: usize) {
    let input = || input::<Vec<_>>(n);
    let filter_map = |x: String| x.starts_with('3').then_some(x);
    let filter = |x: &String| x.ends_with('3');
    let flat_map = |x: String| x.chars().map(|x| x.to_string()).collect::<Vec<_>>();

    let par = input().into_par().num_threads(nt).chunk_size(chunk);
    let par = par.filter_map(filter_map).filter(filter).flat_map(flat_map);
    let vec = ConcurrentVec::new();
    par.inspect(|x| {
        _ = vec.push(x.clone());
    })
    .count();

    assert_eq!(
        sorted(vec.to_vec()),
        sorted(
            input()
                .into_iter()
                .filter_map(filter_map)
                .filter(filter)
                .flat_map(flat_map)
                .collect()
        )
    );
}
