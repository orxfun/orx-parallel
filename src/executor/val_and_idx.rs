pub struct ValIdx<T> {
    pub val: T,
    pub idx: usize,
}

impl<T> ValIdx<T> {
    #[inline(always)]
    pub fn new(val: T, idx: usize) -> Self {
        Self { val, idx }
    }
}
