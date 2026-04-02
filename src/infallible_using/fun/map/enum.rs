use crate::infallible_using::fun::map::fn_trait::MapU;

#[derive(Clone, Copy)]
pub struct MapUEnum<M: MapU>(M);

impl<M: MapU> MapUEnum<M> {
    pub fn new(m: M) -> Self {
        Self(m)
    }
}

impl<M: MapU> MapU for MapUEnum<M> {
    type I = (usize, M::I);

    type O = (usize, M::O);

    type U = M::U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, (idx, i): Self::I) -> Self::O {
        (idx, self.0.map(u, i))
    }
}
