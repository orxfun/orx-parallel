#[test]
fn for_doc() {
    use orx_parallel::*;

    struct ThreadData {
        sum: usize,
    }

    // define how to create thread-local variables
    let mut data = UseVec::new(|_th_idx| ThreadData { sum: 0 });

    (0..100_000)
        .into_par() // ← mutably lend it to parallel iterator
        .use_vec(&mut data)
        .for_each(|d, x| d.sum += x); // ← d: &mut ThreadData

    let results: Vec<ThreadData> = data.into_vec(); // ← get created vars back
}
