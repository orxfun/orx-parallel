use crate::ParThreadPool;

pub fn sort<P, T>(pool: &mut P, slice: &mut [T])
where
    P: ParThreadPool,
    T: Ord,
{
    slice.sort();
}
