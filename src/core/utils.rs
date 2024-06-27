pub fn maybe_reduce<T, R>(reduce: R, a: Option<T>, b: Option<T>) -> Option<T>
where
    R: Fn(T, T) -> T,
{
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => Some(reduce(a, b)),
    }
}
