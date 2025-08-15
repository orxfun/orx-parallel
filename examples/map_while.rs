use orx_parallel::*;

const N: usize = 10_000;
const IDX_BAD_INPUT: [usize; 4] = [1900, 4156, 6777, 5663];

fn good_input() -> Vec<String> {
    (0..N).map(|x| x.to_string()).collect()
}

fn bad_input() -> Vec<String> {
    (0..N)
        .map(|x| match IDX_BAD_INPUT.contains(&x) {
            true => format!("{x}!"),
            false => x.to_string(),
        })
        .collect()
}

fn main() {
    let result: Vec<_> = good_input()
        .par()
        .map_while(|x| x.parse::<usize>().ok())
        .collect();
    assert_eq!(result, (0..N).collect::<Vec<_>>());

    let result: Vec<_> = bad_input()
        .par()
        .map_while(|x| x.parse::<usize>().ok()) // maps until 1900
        .collect();
    assert_eq!(result, (0..IDX_BAD_INPUT[0]).collect::<Vec<_>>());
}
