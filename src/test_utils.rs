#[cfg(not(miri))]
pub const N: &[usize] = &[8025, 42735];
#[cfg(not(miri))]
pub const NT: &[usize] = &[1, 2, 4];
#[cfg(not(miri))]
pub const CHUNK: &[usize] = &[1, 64, 1024];

#[cfg(miri)]
pub const N: &[usize] = &[57];
#[cfg(miri)]
pub const NT: &[usize] = &[2];
#[cfg(miri)]
pub const CHUNK: &[usize] = &[4];

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
