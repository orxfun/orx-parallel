use super::generalized_values::Atom;
use crate::Maybe;

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
pub fn maybe_has_value<O, M: Maybe<O>>(maybe: &M) -> bool {
    maybe.has_value()
}

#[inline(always)]
pub fn map_maybe_value<O, M: Maybe<O>>(maybe: M) -> O {
    maybe.unwrap()
}
