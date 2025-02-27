use test_case::test_matrix;

#[cfg(miri)]
const N: [usize; 2] = [37, 125];
#[cfg(not(miri))]
const N: [usize; 2] = [1025, 4735];

fn new_vec(n: usize, elem: impl Fn(usize) -> String) -> Vec<String> {
    let mut vec = Vec::with_capacity(n + 17);
    for i in 0..n {
        vec.push(elem(i));
    }
    vec
}

#[test_matrix([0, 1, N[0], N[1]], [1, 2, 4], [1, 64, 1024])]
fn empty_collect(n: usize, nt: usize, chunk: usize) {
    //
}
