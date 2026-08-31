use crate::Vec2;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Doubling, Linear, Recursive, SplitVec};

pub trait Collectable<O>: IntoIterator<Item = O> + Send {
    fn col_empty() -> Self;

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self;

    fn col_len(&self) -> usize;

    fn col_reserve(&mut self, additional: usize);

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>);
}

// vec

impl<O: Send> Collectable<O> for alloc::vec::Vec<O> {
    fn col_empty() -> Self {
        Self::new()
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        iter.into_iter().collect()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.extend(iter);
    }
}

// btree-set

impl<O: Send + Ord> Collectable<O> for alloc::collections::BTreeSet<O> {
    fn col_empty() -> Self {
        Self::new()
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        iter.into_iter().collect()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.extend(iter);
    }
}

// vec2

impl<O: Send> Collectable<O> for Vec2<O> {
    fn col_empty() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        iter.into_iter().collect()
    }

    fn col_len(&self) -> usize {
        self.inner.iter().map(|x| x.len()).sum()
    }

    fn col_reserve(&mut self, _: usize) {}

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.extend(iter);
    }
}

// split-vec - doubling

impl<O: Send> Collectable<O> for SplitVec<O, Doubling> {
    fn col_empty() -> Self {
        Self::with_doubling_growth()
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        iter.into_iter().collect()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.extend(iter);
    }
}

// split-vec - recursive

impl<O: Send> Collectable<O> for SplitVec<O, Recursive> {
    fn col_empty() -> Self {
        Self::with_recursive_growth()
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        iter.into_iter().collect()
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.append(iter.into_iter().collect::<alloc::vec::Vec<_>>());
    }
}

// split-vec - linear

impl<O: Send> Collectable<O> for SplitVec<O, Linear> {
    fn col_empty() -> Self {
        Self::with_linear_growth(10)
    }

    fn col_from_iter(iter: impl IntoIterator<Item = O>) -> Self {
        let mut vec = Self::col_empty();
        vec.extend(iter);
        vec
    }

    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}

    fn col_extend(&mut self, iter: impl IntoIterator<Item = O>) {
        self.extend(iter);
    }
}
