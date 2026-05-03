use orx_parallel::*;

const LOG: &[&str] = &[
    "INFO  service started",
    "DEBUG loading config",
    "ERROR failed to connect to database",
    "INFO  retrying connection",
    "ERROR timeout after 30s",
    "INFO  connection restored",
    "WARN  high memory usage",
    "ERROR disk quota exceeded",
];

// Expected line numbers of ERROR entries (0-based).
const ERROR_LINE_NUMBERS: &[usize] = &[2, 4, 7];

/// `enumerate()` can be placed first in the chain.
/// Pairing each line with its number before filtering lets the result carry
/// the original position of every error line.
fn enumerate_then_filter() {
    let error_positions: Vec<usize> = LOG
        .par()
        .enumerate()
        .filter(|(_, line)| line.starts_with("ERROR"))
        .map(|(pos, _)| pos)
        .collect();

    assert_eq!(error_positions, ERROR_LINE_NUMBERS);
}

/// `enumerate()` can also appear after a `map` step.
/// The index still corresponds to the position in the original input —
/// the preceding `map` is a 1-to-1 transformation and does not shift indices.
fn map_then_enumerate() {
    let error_positions: Vec<usize> = LOG
        .par()
        .map(|line| line.split_whitespace().next().unwrap_or(""))
        .enumerate()
        .filter(|(_, level)| *level == "ERROR")
        .map(|(pos, _)| pos)
        .collect();

    assert_eq!(error_positions, ERROR_LINE_NUMBERS);
}

fn main() {
    enumerate_then_filter();
    map_then_enumerate();
}
