use super::into_iter_ptr_dst::IntoIterPtrDst;
use super::iter_ptr_dst::IterPtrDst;
use super::slice_src::SliceSrc;
use crate::results::ValIdx;
use alloc::vec::Vec;

pub fn seq<T>(results: Vec<Vec<ValIdx<T>>>, mut dst1: Vec<T>, mut dst2: Vec<T>) {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    dst1.reserve(total_len);
    dst2.reserve(total_len);

    let offsets = [dst1.len(), dst2.len()];
    let destinations = [dst1, dst2];

    let mut slices_curr: Vec<_> = results.iter().map(|x| SliceSrc::from_slice(&x)).collect();
    // let mut slices_next = Vec::with_capacity(slices_curr.len() / 2);

    let mut dst_idx = 1;
    while slices_curr.len() > 1 {
        dst_idx = (dst_idx + 1) % 2;

        let mut offset = offsets[dst_idx];

        for pair in slices_curr.chunks(2) {
            match (pair.get(0).copied(), pair.get(1).copied()) {
                (Some(a), Some(b)) => {
                    let (beg, end) = (offset, offset + a.len() + b.len());
                    let dst_slice = &destinations[dst_idx][beg..end];
                    let dst = dst_slice.into_iter_ptr_dst();
                    seq_merge(a, b, dst);
                    offset += a.len() + b.len();

                    // let new_src = SliceSrc::from_slice(dst_slice);
                }
                (Some(a), None) => {
                    let b = SliceSrc::from_slice(&[]);
                    let (beg, end) = (offset, offset + a.len() + b.len());
                    let dst_slice = &destinations[dst_idx][beg..end];
                    let dst = dst_slice.into_iter_ptr_dst();
                    seq_merge(a, b, dst);
                    offset += a.len() + b.len();
                }
                _ => todo!(),
            }
        }

        todo!()
    }

    todo!()
}

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
