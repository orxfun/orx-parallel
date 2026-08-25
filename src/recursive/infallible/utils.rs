use alloc::vec::Vec;
use orx_iterable::CollectionMut;

pub fn into_outer<'a, T>(outer: &'a mut Vec<T>, locals: &mut impl CollectionMut<Item = Vec<T>>) {
    outer.clear();

    let total_len: usize = locals.iter_mut().map(|x| x.len()).sum();
    outer.reserve(total_len);

    let mut dst = outer.as_mut_ptr();
    for local in locals.iter_mut() {
        let (src, len) = (local.as_mut_ptr(), local.len());

        unsafe { src.copy_to_nonoverlapping(dst, len) };
        unsafe { outer.set_len(outer.len() + len) };
        unsafe { local.set_len(0) };

        dst = unsafe { dst.add(len) };
    }
}
