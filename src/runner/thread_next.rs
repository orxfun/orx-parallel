pub enum ThreadNext<T> {
    Found { idx: usize, value: T },
    NotFound,
    Stopped { idx: usize },
}

impl<T> ThreadNext<T> {
    /// Returns the value with the smallest found idx whose idx is less than
    /// the smallest of the stopped indices, if any.
    ///
    /// Returns None if there is no found items before the process stopped.
    pub fn reduce(results: Vec<Self>) -> Option<(usize, T)> {
        let mut idx_bound = usize::MAX;
        let mut result = None;
        for x in results {
            match x {
                Self::Found { idx, value } if idx < idx_bound => {
                    idx_bound = idx;
                    result = Some((idx, value));
                }
                Self::Stopped { idx } if idx < idx_bound => idx_bound = idx,
                _ => {}
            }
        }

        result
    }
}
