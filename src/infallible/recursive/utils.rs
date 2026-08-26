use alloc::vec::Vec;
use orx_iterable::CollectionMut;

pub fn into_outer<'a, T>(
    outer: &'a mut Vec<T>,
    total_len: usize,
    locals: impl IntoIterator<Item = &'a mut Vec<T>>,
) {
    outer.clear();
    outer.reserve(total_len);

    let mut dst = outer.as_mut_ptr();
    for local in locals {
        let (src, len) = (local.as_mut_ptr(), local.len());

        unsafe { src.copy_to_nonoverlapping(dst, len) };
        unsafe { outer.set_len(outer.len() + len) };
        unsafe { local.set_len(0) };

        dst = unsafe { dst.add(len) };
    }
}
