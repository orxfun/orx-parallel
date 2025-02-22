pub fn test_different_params<T>(test: T)
where
    T: Fn(usize, usize),
{
    let params = [
        [1, 1],
        [1, 2],
        [2, 1],
        [2, 2],
        [4, 1024],
        [16, 64],
        [8, 32],
        [100, 1],
        [2, 13],
    ];

    for [num_threads, chunk_size] in params {
        test(num_threads, chunk_size);
    }
}

pub fn test_reduce<T>(test: T)
where
    T: Fn(usize, usize, usize),
{
    let params = [
        [0, 1, 1],
        [0, 2, 2],
        [1, 3, 1],
        [64, 4, 1024],
        [1024, 1, 1],
        [1344, 1, 2],
        [8981, 2, 1],
        [4999, 4, 64],
        [7850, 8, 8],
    ];

    for [len, num_threads, chunk_size] in params {
        test(len, num_threads, chunk_size);
    }
}

#[allow(dead_code)]
pub fn some_if<T, F: Fn(&T) -> bool>(value: T, condition: F) -> Option<T> {
    match condition(&value) {
        true => Some(value),
        false => None,
    }
}

#[allow(dead_code)]
pub fn ok_if<T, F: Fn(&T) -> bool>(value: T, condition: F) -> Result<T, String> {
    match condition(&value) {
        true => Ok(value),
        false => Err("not-okay".to_string()),
    }
}
