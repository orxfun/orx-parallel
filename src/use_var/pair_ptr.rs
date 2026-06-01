pub struct PairPtr<U, V> {
    u: *mut U,
    v: *mut V,
}

unsafe impl<U, V> Send for PairPtr<U, V> {}

impl<U, V> PairPtr<U, V> {
    pub fn new(u: *mut U, v: *mut V) -> Self {
        Self { u, v }
    }

    pub fn u_mut(&mut self) -> &mut U {
        unsafe { &mut *self.u }
    }
}
