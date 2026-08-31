use crate::Vec2;

pub trait Collectable<O>: Extend<O> + FromIterator<O> + IntoIterator<Item = O> + Send {
    fn col_empty() -> Self;

    fn col_len(&self) -> usize;

    fn col_reserve(&mut self, additional: usize);
}

// vec

impl<O: Send> Collectable<O> for alloc::vec::Vec<O> {
    fn col_empty() -> Self {
        Self::new()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

// btree-set

impl<O: Send + Ord> Collectable<O> for alloc::collections::BTreeSet<O> {
    fn col_empty() -> Self {
        Self::new()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}
}

// vec2

impl<O: Send> Collectable<O> for Vec2<O> {
    fn col_empty() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn col_len(&self) -> usize {
        self.inner.iter().map(|x| x.len()).sum()
    }

    fn col_reserve(&mut self, _: usize) {}
}
