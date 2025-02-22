#[inline(always)]
pub const fn map_self<T>(x: T) -> T {
    x
}

#[inline(always)]
pub const fn no_filter<T>(_: &T) -> bool {
    true
}
