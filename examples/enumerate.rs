pub use orx_parallel::*;

fn start_by_enumerate() {
    let inputs: Vec<i32> = (7..4242).collect();
    let expected_sum_indices: usize = (0..inputs.len()).sum();
    let expected_sum_values = inputs.iter().sum::<i32>() + 100 * inputs.len() as i32;

    let (sum_indices, sum_values) = inputs
        .into_par()
        .enumerate()
        .map(|(idx, val)| (idx, val + 100))
        .reduce(|agg, (idx, value)| (agg.0 + idx, agg.1 + value))
        .unwrap_or((0, 0));

    assert_eq!(sum_indices, expected_sum_indices);
    assert_eq!(sum_values, expected_sum_values);
}

fn map_then_enumerate() {
    // let inputs: Vec<i32> = (7..4242).collect();
    // let expected_sum_indices: usize = (0..inputs.len()).sum();
    // let expected_sum_values = inputs.iter().sum::<i32>() + 100 * inputs.len() as i32;

    // let (sum_indices, sum_values) = inputs
    //     .into_par()
    //     .map(|x| x + 100)
    //     .enumerate()
    //     .reduce(|agg, (idx, value)| (agg.0 + idx, agg.1 + value))
    //     .unwrap_or((0, 0));

    // assert_eq!(sum_indices, expected_sum_indices);
    // assert_eq!(sum_values, expected_sum_values);
}

fn main() {
    start_by_enumerate();
    map_then_enumerate();
}
