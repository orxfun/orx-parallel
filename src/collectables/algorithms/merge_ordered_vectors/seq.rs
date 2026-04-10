use super::iter_ptr_dst::IterPtrDst;
use super::slice_src::SliceSrc;

pub fn seq_merge<'a, T: 'a, D>(left: SliceSrc<'a, T>, right: SliceSrc<'a, T>, mut dst: D)
where
    D: IterPtrDst<'a, T>,
{
    let mut left = left.into_ptr_iter();
    let mut right = right.into_ptr_iter();

    match (left.current_idx(), right.current_idx()) {
        (Some(mut idx_l), Some(mut idx_r)) => {
            loop {
                match idx_l < idx_r {
                    true => {
                        // SAFETY: left still has at least one elem, so must `dst`
                        unsafe { dst.write_one_from(&mut left) };
                        match left.current_idx() {
                            Some(x) => idx_l = x,
                            None => {
                                unsafe { dst.write_rest_from(right) };
                                break;
                            }
                        }
                    }
                    false => {
                        // SAFETY: right still has at least one elem, so must `dst`
                        unsafe { dst.write_one_from(&mut right) };

                        match right.current_idx() {
                            Some(x) => idx_r = x,
                            None => {
                                unsafe { dst.write_rest_from(left) };
                                break;
                            }
                        }
                    }
                }
            }
        }
        (None, None) => {}
        (None, _) => unsafe { dst.write_rest_from(right) },
        (_, None) => unsafe { dst.write_rest_from(left) },
    }
}
