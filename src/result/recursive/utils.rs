use crate::infallible::Xap;
use alloc::vec::Vec;

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

/// Collects `x1.xap(i)` into successful `M` values, or `Err(e)` as soon as any output fails.
pub fn split<M, E, X1>(x1: &X1, i: X1::I) -> Result<Vec<M>, E>
where
    X1: Xap<O = Result<M, E>>,
{
    let mut ms = Vec::new();
    for o in x1.xap(i) {
        ms.push(o?);
    }
    Ok(ms)
}
