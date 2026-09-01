pub trait ParExtend<T>: Default + Extend<T> {
    fn len(&self) -> usize;

    fn push_one(&mut self, value: T);
}

pub trait Contiguous<T>: ParExtend<T> {
    fn reserve(&mut self, additional: usize);

    fn capacity(&self) -> usize;

    unsafe fn ptr(&mut self, idx: usize) -> *mut T;

    unsafe fn set_len(&mut self, new_len: usize);
}
