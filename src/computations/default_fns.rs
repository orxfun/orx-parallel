use super::generalized_values::Atom;
use std::ops::Add;

#[inline(always)]
pub fn map_self<T>(input: T) -> T {
    input
}

#[inline(always)]
pub fn map_self_atom<T>(input: T) -> Atom<T> {
    Atom(input)
}

#[inline(always)]
pub fn map_count<T>(_: T) -> usize {
    1
}

#[inline(always)]
pub fn map_copy<T: Copy>(x: &T) -> T {
    *x
}

#[inline(always)]
pub fn map_clone<T: Clone>(x: &T) -> T {
    x.clone()
}

#[inline(always)]
pub fn reduce_sum<T>(a: T, b: T) -> T
where
    T: Add<T, Output = T>,
{
    a + b
}

#[inline(always)]
pub fn reduce_unit(_: (), _: ()) {}

// using

#[inline(always)]
pub fn u_map_self<U, T>(_: &mut U, input: T) -> T {
    input
}

#[inline(always)]
pub fn u_map_self_atom<U, T>(_: &mut U, input: T) -> Atom<T> {
    Atom(input)
}

#[inline(always)]
pub fn u_map_copy<U, T: Copy>(_: &mut U, x: &T) -> T {
    *x
}

#[inline(always)]
pub fn u_map_clone<U, T: Clone>(_: &mut U, x: &T) -> T {
    x.clone()
}

#[inline(always)]
pub fn u_map_count<U, T>(_: &mut U, _: T) -> usize {
    1
}
