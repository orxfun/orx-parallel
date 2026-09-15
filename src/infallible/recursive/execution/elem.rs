use core::cmp::Ordering;

pub struct ElemOut<T> {
    pub value: T,
    pub depth: usize,
    pub width: usize,
}

impl<T> ElemOut<T> {
    pub fn new(value: T, depth: usize, width: usize) -> Self {
        Self {
            value,
            depth,
            width,
        }
    }
}

impl<T> PartialEq for ElemOut<T> {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth && self.width == other.width
    }
}

impl<T> Eq for ElemOut<T> {}

impl<T> PartialOrd for ElemOut<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for ElemOut<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.depth.cmp(&other.depth) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.width.cmp(&other.width)
    }
}

pub struct ElemIn<T> {
    pub value: T,
    pub parent_idx: usize,
    pub child_idx: usize,
}

impl<T> ElemIn<T> {
    pub fn new(value: T, parent_idx: usize, child_idx: usize) -> Self {
        Self {
            value,
            parent_idx,
            child_idx,
        }
    }

    pub fn normalize_parent_indices(elements: &mut [Self]) {
        if let Some(max_width) = elements.iter().map(|x| x.child_idx).max() {
            let depth_coef = max_width + 1;
            for elem in elements.iter_mut() {
                elem.parent_idx = elem.parent_idx * depth_coef + elem.child_idx;
            }
        }
    }
}
