use crate::collectables::par_extend::ParExtend;
use alloc::collections::BTreeSet;

impl<T: Ord> ParExtend<T> for BTreeSet<T> {
    fn len(&self) -> usize {
        BTreeSet::len(self)
    }

    fn push_one(&mut self, value: T) {
        self.insert(value);
    }
}
