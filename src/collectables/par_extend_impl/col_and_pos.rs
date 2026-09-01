use crate::collectables::par_extend_impl::idx_len::IdxLen;
use alloc::vec::Vec;

pub struct ColAndPos<C> {
    pub values: C,
    pub positions: Vec<IdxLen>,
}

impl<C: Default> Default for ColAndPos<C> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            positions: Vec::new(),
        }
    }
}
