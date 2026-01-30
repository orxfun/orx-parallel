use super::core::SliceIterCore;

pub struct SliceIterRef<'a, T: 'a>(SliceIterCore<'a, T>);

impl<'a, T: 'a> Iterator for SliceIterRef<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: SliceIterCore::next returns a valid pointer.
        self.0.next().map(|x| unsafe { &*x })
    }
}
