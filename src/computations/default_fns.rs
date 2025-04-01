use std::ops::Add;

use super::generalized_values::Atom;

#[inline(always)]
pub fn map_self<T>(input: T) -> T {
    input
}

#[inline(always)]
pub fn map_self_atom<T>(input: T) -> Atom<T> {
    Atom(input)
}

#[inline(always)]
pub fn filter_true<T>(_: &T) -> bool {
    true
}

#[inline(always)]
pub fn map_count<T>(_: T) -> usize {
    1
}

#[inline(always)]
pub fn reduce_sum<T>(a: T, b: T) -> T
where
    T: Add<T, Output = T>,
{
    a + b
}
