use crate::infallible::fun::Map;

#[derive(Clone, Copy)]
pub struct MapEnum<M: Map>(M);

impl<M: Map> MapEnum<M> {
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
