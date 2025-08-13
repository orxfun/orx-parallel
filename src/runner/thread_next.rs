pub enum ThreadNext<T, E> {
    Found { idx: usize, value: T },
    NotFound,
    Stopped { idx: usize, stop_with: E },
}

impl<T, E> ThreadNext<T, E> {
    fn found_or_stopped_idx(&self) -> Option<usize> {
        match self {
            Self::Found { idx, value: _ } => Some(*idx),
            Self::Stopped { idx, stop_with: _ } => Some(*idx),
            _ => None,
        }
    }

    pub fn into_found_value(self) -> Option<T> {
        match self {
            Self::Found { idx: _, value } => Some(value),
            _ => None,
        }
    }

    /// Returns the value with the smallest found idx whose idx is less than
    /// the smallest of the stopped indices, if any.
    ///
    /// Returns None if there is no found items before the process stopped.
    pub fn reduce(results: Vec<Self>) -> Self {
        let mut result = Self::NotFound;
        let mut idx_bound = usize::MAX;
        for x in results {
            if let Some(idx) = x.found_or_stopped_idx()
                && idx < idx_bound
            {
                idx_bound = idx;
                result = x;
            }
        }

        result
    }
}
