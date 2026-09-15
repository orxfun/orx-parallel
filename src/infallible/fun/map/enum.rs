use crate::infallible::fun::Map;

#[derive(Clone, Copy)]
/// Map adapter that preserves an input index.
pub struct MapEnum<M: Map>(M);

impl<M: Map> MapEnum<M> {
    /// Creates an enumerating map adapter.
    pub fn new(m: M) -> Self {
        Self(m)
    }
}

impl<M: Map> Map for MapEnum<M> {
    type I = (usize, M::I);

    type O = (usize, M::O);

    #[inline(always)]
    fn map(&self, (idx, i): Self::I) -> Self::O {
        (idx, self.0.map(i))
    }
}
