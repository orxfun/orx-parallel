use crate::infallible_use::fun::map::fn_trait::UMap;

#[derive(Clone, Copy)]
pub struct UMapEnum<M: UMap>(M);

impl<M: UMap> UMapEnum<M> {
    pub fn new(m: M) -> Self {
        Self(m)
    }
}

impl<M: UMap> UMap for UMapEnum<M> {
    type I = (usize, M::I);

    type O = (usize, M::O);

    type U = M::U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, (idx, i): Self::I) -> Self::O {
        (idx, self.0.map(u, i))
    }
}
