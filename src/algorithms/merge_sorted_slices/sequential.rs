use crate::algorithms::data_structures::Slice;

pub fn merge_sorted_slices<'a, T, F>(
    is_leq: F,
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target: Slice<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
}
