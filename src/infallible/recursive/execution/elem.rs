use core::cmp::Ordering;

pub struct Elem<T> {
    pub value: T,
    pub depth: usize,
    pub width: usize,
}

impl<T> Elem<T> {
    pub fn new(value: T, depth: usize, width: usize) -> Self {
        Self {
            value,
            depth,
            width,
        }
    }

    pub fn normalize_depths(elements: &mut [Self]) {
        if let Some(max_width) = elements.iter().map(|x| x.width).max() {
            let depth_coef = max_width + 1;
            for elem in elements {
                elem.depth = elem.depth * depth_coef + elem.width;
            }
        }
    }
}

impl<T> PartialEq for Elem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth && self.width == other.width
    }
}

impl<T> Eq for Elem<T> {}

impl<T> PartialOrd for Elem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Elem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.depth.cmp(&other.depth) {
            Ordering::Equal => {}
            ord => return ord,
        }
        debug_assert_ne!(self.width, other.width);
        self.width.cmp(&other.width)
    }
}
