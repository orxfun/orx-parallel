use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;

pub fn into_outer<'a, T, M, F>(outer: &'a mut Vec<T>, locals: &'a mut [M], local_data: F)
where
    F: Fn(&'a mut M) -> &'a mut Vec<T>,
{
    let total_len: usize = {
        let locals = unsafe { &mut *(locals as *mut [M]) };
        locals.iter_mut().map(|x| local_data(x).len()).sum()
    };
    outer.reserve(total_len);

    let mut dst = unsafe { outer.as_mut_ptr().add(outer.len()) };
    for local in locals.iter_mut() {
        let local = local_data(local);
        let (src, len) = (local.as_mut_ptr(), local.len());

        unsafe { src.copy_to_nonoverlapping(dst, len) };
        unsafe { outer.set_len(outer.len() + len) };
        unsafe { local.set_len(0) };

        dst = unsafe { dst.add(len) };
    }
}
