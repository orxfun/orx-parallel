use super::core::SliceIterCore;

pub struct SliceIterRef<'a, T: 'a>(SliceIterCore<'a, T>);

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
