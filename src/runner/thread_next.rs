pub enum ThreadNext<T> {
    Found { idx: usize, value: T },
    NotFound,
    Stopped { idx: usize },
}

impl<T> ThreadNext<T> {
    pub fn reduce(results: Vec<Self>) -> Option<(usize, T)> {
        let mut stopped_idx = usize::MAX;

        for result in &results {
            match result {
                Self::Stopped { idx } => stopped_idx = core::cmp::min(stopped_idx, *idx),
                _ => {}
            }
        }

        let mut found_idx = usize::MAX;
        let mut result = None;
        for x in results {
            match x {
                Self::Found { idx, value } if idx < stopped_idx && idx < found_idx => {
                    found_idx = idx;
                    result = Some((idx, value));
                }
                _ => {}
            }
        }

        result
    }
}
