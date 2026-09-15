use crate::extendable::par_extend_impl::utils::IdxLen;
use alloc::vec::Vec;

/// A collected payload together with its chunk positions.
pub struct ColAndPos<C> {
    /// Collected values.
    pub values: C,
    /// Source positions for each collected chunk.
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
