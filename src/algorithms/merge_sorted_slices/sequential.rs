use crate::algorithms::data_structures::{Slice, SliceMut};

pub fn merge_sorted_slices<'a, T: 'a, F>(
    is_leq: F,
    left: &Slice<'a, T>,
    right: &Slice<'a, T>,
    target: &mut SliceMut<'a, T>,
) where
    F: Fn(&T, &T) -> bool,
{
    debug_assert_eq!(left.len() + right.len(), target.len());

    let mut dst = target.iter_as_dst();

    match (left.len(), right.len()) {
        (0, _) => unsafe { dst.write_remaining_from(right) },
        (_, 0) => unsafe { dst.write_remaining_from(left) },
        _ => {
            let mut left = left.iter_over_ptr();
            let mut right = right.iter_over_ptr();

            loop {
                let (a, b) = unsafe {
                    let l = left.current_unchecked();
                    let r = right.current_unchecked();

                    match is_leq(l, r) {
                        true => (&mut left, &mut right),
                        false => (&mut right, &mut left),
                    }
                };

                unsafe { dst.write_one_unchecked(a.next_unchecked()) };

                if a.is_finished() {
                    unsafe { dst.write_remaining_from(&b.remaining_into_slice()) };
                    break;
                }
            }
        }
    }
}
