pub fn min_opt_idx(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(core::cmp::min(a, b)),
        (None, b) => b,
        _ => a,
    }
}
