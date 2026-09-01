use crate::collectables::par_extend::ParExtend;
use alloc::vec::Vec;

impl<T> ParExtend<T> for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn push_one(&mut self, value: T) {
        self.push(value);
    }
}
