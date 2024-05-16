use crate::Par;
use orx_concurrent_iter::{ClonedConIterOfSlice, ConIterOfSlice};

impl<'a, T: Send + Sync + Clone> Par<ConIterOfSlice<'a, T>> {
    pub fn cloned(self) -> Par<ClonedConIterOfSlice<'a, T>> {
        Par::map_data(self, |x| x.cloned())
    }
}
