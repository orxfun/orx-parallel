use std::ops::Add;

#[inline(always)]
pub fn u_map_self<U, T>(_: &mut U, input: T) -> T {
    input
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

#[inline(always)]
pub fn u_reduce_sum<U, T>(_: &mut U, a: T, b: T) -> T
where
    T: Add<T, Output = T>,
{
    a + b
}

#[inline(always)]
pub fn u_reduce_unit<U>(_: &mut U, _: (), _: ()) {}
