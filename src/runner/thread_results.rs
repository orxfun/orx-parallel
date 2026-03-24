use alloc::vec::Vec;

pub struct ThreadResults<T> {
    p: *mut T,
    len: usize,
    must_drop: bool,
}

impl<T> ThreadResults<T> {
    pub fn new(len: usize) -> Self {
        let mut vec = Vec::with_capacity(len);
        let p = vec.as_mut_ptr();
        let must_drop = true;
        core::mem::forget(vec);
        Self { p, len, must_drop }
    }

    pub unsafe fn into_vec(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.p, self.len, self.len) }
    }

    pub fn write(&self, th_idx: usize, value: T) {
        let p = unsafe { self.p.add(th_idx) };
        unsafe { p.write(value) };
    }
}

impl<T> Drop for ThreadResults<T> {
    fn drop(&mut self) {
        if self.must_drop {
            let _vec = unsafe { Vec::from_raw_parts(self.p, self.len, self.len) };
            self.p = core::ptr::null_mut();
            self.must_drop = false;
        }
    }
}
