use alloc::vec::Vec;

pub fn inputs_into_outer<'a, T>(
    outer: &'a mut Vec<T>,
    total_len: usize,
    locals: impl IntoIterator<Item = &'a mut Vec<T>>,
) {
    debug_assert!(outer.is_empty());
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

pub fn outputs_into_outer<'a, T>(
    outer: &'a mut Vec<T>,
    total_len: usize,
    locals: impl IntoIterator<Item = &'a mut Vec<T>>,
) {
    outer.reserve(total_len);

    let mut dst = unsafe { outer.as_mut_ptr().add(outer.len()) };
    for local in locals {
        let (src, len) = (local.as_mut_ptr(), local.len());

        unsafe { src.copy_to_nonoverlapping(dst, len) };
        unsafe { outer.set_len(outer.len() + len) };
        unsafe { local.set_len(0) };

        dst = unsafe { dst.add(len) };
    }
}
