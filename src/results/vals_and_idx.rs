use alloc::vec::Vec;

/// Marks the starting input index and number of consecutive elements.
pub struct IdxLen {
    pub idx: usize,
    pub len: usize,
}

/// Collected values and associated indices and lengths of the collected elements.
pub struct ValsAndIdx<T> {
    pub values: Vec<T>,
    pub positions: Vec<IdxLen>,
}

impl<T> ValsAndIdx<T> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            positions: Vec::new(),
        }
    }

    #[inline]
    pub fn extend(&mut self, idx: usize, values: impl IntoIterator<Item = T>) {
        let len_begin = self.values.len();
        self.values.extend(values);

        let len_added = self.values.len() - len_begin;
        self.positions.push(IdxLen {
            idx,
            len: len_added,
        });
    }
}
