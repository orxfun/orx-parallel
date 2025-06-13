/// Input size.
#[cfg(not(miri))]
pub const N: &[usize] = &[8025, 42735];
/// Number of threads.
#[cfg(not(miri))]
pub const NT: &[usize] = &[0, 1, 2, 4];
/// Chunk size.
#[cfg(not(miri))]
pub const CHUNK: &[usize] = &[0, 1, 64, 1024];

/// Input size.
#[cfg(miri)]
pub const N: &[usize] = &[57];
/// Number of threads.
#[cfg(miri)]
pub const NT: &[usize] = &[1, 2];
/// Chunk size.
#[cfg(miri)]
pub const CHUNK: &[usize] = &[4];

/// Test all combinations of the settings with the `test` method.
pub fn test_n_nt_chunk<T>(n: &[usize], nt: &[usize], chunk: &[usize], test: T)
where
    T: Fn(usize, usize, usize),
{
    for n in n {
        for nt in nt {
            for chunk in chunk {
                test(*n, *nt, *chunk);
            }
        }
    }
}
