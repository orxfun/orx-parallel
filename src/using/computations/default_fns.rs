use crate::computations::Atom;

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
