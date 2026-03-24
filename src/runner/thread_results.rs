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

    /// # SAFETY
    ///
    /// The following must be satisfied to call this method safely:
    ///
    /// - (i) `write` must be called with all indices between `0..self.len` beforehand.
    pub unsafe fn into_vec(mut self) -> Vec<T> {
        self.must_drop = false;

        // SAFETY: by (i) all elements of the vector are initialized
        unsafe { Vec::from_raw_parts(self.p, self.len, self.len) }
    }

    /// # SAFETY:
    ///
    /// - (i) `th_idx` must be a valid index; i.e., `th_idx < self.len`
    /// - (ii) This method must be called exactly once by one of the threads used in parallel
    ///   computation.
    pub fn write(&self, th_idx: usize, value: T) {
        // SAFETY: by (i), `p` is in bounds
        let p = unsafe { self.p.add(th_idx) };
        // SAFETY: by (ii), there exists no race condition
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
