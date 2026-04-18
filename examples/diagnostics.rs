use orx_parallel::*;

fn main() {
    let n = 1_000_000;
    let n = 100;

    let sum = (0..n)
        .par()
        .map(|i| 2 * i)
        .flat_map(|i| match i.is_multiple_of(2) {
            true => [i, i + 1, i + 2, i + 3, i + 4],
            false => [i, i + 1, usize::MAX, usize::MAX, usize::MAX],
        })
        .filter_map(|i| (i < usize::MAX).then_some(i))
        .reduce(|a, b| a + b)
        .unwrap();
    println!("sum with default runner = {sum}");

    let sum = (0..n)
        .par()
        .runner_with_diagnostics()
        .map(|i| 2 * i)
        .flat_map(|i| match i.is_multiple_of(2) {
            true => [i, i + 1, i + 2, i + 3, i + 4],
            false => [i, i + 1, usize::MAX, usize::MAX, usize::MAX],
        })
        .filter_map(|i| (i < usize::MAX).then_some(i))
        .reduce(|a, b| a + b)
        .unwrap();
    println!("sum using runner with diagnostics = {sum}");
}
