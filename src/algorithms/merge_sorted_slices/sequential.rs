use crate::algorithms::data_structures::Slice;

pub fn merge_sorted_slices<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut Slice<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    let mut dst = target.iter_as_dst();
}
