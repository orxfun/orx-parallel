use orx_parallel::*;

fn main() {
    let mut input: Vec<_> = (0..4242).collect();
    let initial_sum: i32 = input.iter().sum();

    let mid_sum = input.par_drain(1000..2000).reduce(|a, b| a + b).unwrap();

    assert_eq!(mid_sum, (1000..2000).sum());
    assert_eq!(input.len(), 4242 - 1000);

    let remaining_sum: i32 = input.iter().sum();
    assert_eq!(initial_sum, mid_sum + remaining_sum);
}
