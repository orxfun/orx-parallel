use crate::runner::ParRunner;
use alloc::vec::Vec;

#[allow(dead_code)]
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
    for local in locals {
        let local = local_data(local);
        let (src, len) = (local.as_mut_ptr(), local.len());

        unsafe { src.copy_to_nonoverlapping(dst, len) };
        unsafe { outer.set_len(outer.len() + len) };
        unsafe { local.set_len(0) };

        dst = unsafe { dst.add(len) };
    }
}

pub fn into_outer_par<'a, T, M, F, R>(
    outer: &'a mut Vec<T>,
    locals: &'a mut [M],
    local_data: F,
    runner: &mut R,
) where
    R: ParRunner,
    F: Fn(&'a mut M) -> &'a mut Vec<T>,
{
    use crate::*;
    struct Cpy<T> {
        src: *mut T,
        dst: *mut T,
        len: usize,
    }
    unsafe impl<T> Send for Cpy<T> {}
    unsafe impl<T> Sync for Cpy<T> {}

    let total_len: usize = {
        let locals = unsafe { &mut *(locals as *mut [M]) };
        locals.iter_mut().map(|x| local_data(x).len()).sum()
    };
    outer.reserve(total_len);

    let ops = {
        let locals = unsafe { &mut *(locals as *mut [M]) };
        let mut ptrs = Vec::with_capacity(locals.len());
        let mut begin = outer.len();
        for local in locals {
            let local = local_data(local);
            let (src, len) = (local.as_mut_ptr(), local.len());
            let dst = unsafe { outer.as_mut_ptr().add(begin) };
            ptrs.push(Cpy { src, dst, len });
            begin += len;
            unsafe { local.set_len(0) };
        }
        ptrs
    };

    let par = ops.into_par().runner(runner);
    par.for_each(|op: Cpy<T>| {
        let Cpy { src, dst, len } = op;
        unsafe { src.copy_to_nonoverlapping(dst, len) };
    });

    unsafe { outer.set_len(outer.len() + total_len) };
}
