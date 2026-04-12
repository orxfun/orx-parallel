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

        let len = self.values.len() - len_begin;
        self.positions.push(IdxLen { idx, len });
    }

    /// Returns the first observed error if any; returns None if all succeeds.
    #[inline]
    pub fn extend_res<E>(
        &mut self,
        idx: usize,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Option<E> {
        let len_begin = self.values.len();
        for x in values {
            match x {
                Ok(x) => self.values.push(x),
                Err(e) => {
                    let len = self.values.len() - len_begin;
                    self.positions.push(IdxLen { idx, len });
                    return Some(e);
                }
            }
        }

        let len = self.values.len() - len_begin;
        self.positions.push(IdxLen { idx, len });

        None
    }

    /// Returns `true` if at least one element is None; returns `false` if all are Some variant.
    #[inline]
    pub fn extend_opt(&mut self, idx: usize, values: impl IntoIterator<Item = Option<T>>) -> bool {
        let len_begin = self.values.len();
        for x in values {
            match x {
                Some(x) => self.values.push(x),
                None => {
                    let len = self.values.len() - len_begin;
                    self.positions.push(IdxLen { idx, len });
                    return true;
                }
            }
        }

        let len = self.values.len() - len_begin;
        self.positions.push(IdxLen { idx, len });

        false
    }
}
