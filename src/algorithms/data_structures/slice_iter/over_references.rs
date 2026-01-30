use super::core::SliceIterCore;
use crate::algorithms::data_structures::slice::Slice;

pub struct SliceIterRef<'a, T: 'a>(SliceIterCore<'a, T>);

impl<'a, T: 'a> From<&Slice<'a, T>> for SliceIterRef<'a, T> {
    fn from(value: &Slice<'a, T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> Iterator for SliceIterRef<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: SliceIterCore::next returns a valid pointer.
        self.0.next().map(|x| unsafe { &*x })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl<'a, T: 'a> ExactSizeIterator for SliceIterRef<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: 'a> SliceIterRef<'a, T> {
    pub fn peek(&self) -> Option<&'a T> {
        // SAFETY: SliceIterCore::peek returns a valid pointer.
        self.0.peek().map(|x| unsafe { &*x })
    }
}
