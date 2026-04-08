use crate::infallible_use::fun::map::fn_trait::Map;

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

    type U = M::U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, (idx, i): Self::I) -> Self::O {
        (idx, self.0.map(u, i))
    }
}
