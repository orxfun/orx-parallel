use orx_parallel::*;
use test_case::test_matrix;

#[test_matrix([193], [0, 1], [0, 47])]
fn chain_known_known(n: usize, nt: usize, chunk: usize) {
    let a: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let b: Vec<_> = (0..n).map(|x| (n + x).to_string()).collect();

    let c: Vec<_> = a
        .par()
        .num_threads(nt)
        .chunk_size(chunk)
        .chain(&b)
        .collect();
    assert_eq!(c.len(), 2 * n);
    assert_eq!(c, a.iter().chain(&b).collect::<Vec<_>>());
}

#[test_matrix([193], [0, 1], [0, 47])]
fn chain_known_unknown(n: usize, nt: usize, chunk: usize) {
    let a: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let b: Vec<_> = (0..n)
        .map(|x| (n + x).to_string())
        .filter(|x| !x.starts_with('x'))
        .collect();

    let c: Vec<_> = a
        .par()
        .num_threads(nt)
        .chunk_size(chunk)
        .chain(&b)
        .collect();
    assert_eq!(c.len(), 2 * n);
    assert_eq!(c, a.iter().chain(&b).collect::<Vec<_>>());
}
