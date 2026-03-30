use crate::infallible::{fun::map::FnCloned, xap::Xap};

pub trait XapCloned<'a, O: 'a + Clone>: Xap<O = &'a O> {
    fn cloned(self) -> Self::Mapped<FnCloned<'a, O>>;
}

impl<'a, O: 'a + Clone, X: Xap<O = &'a O>> XapCloned<'a, O> for X {
    fn cloned(self) -> Self::Mapped<FnCloned<'a, O>> {
        self.mapped(FnCloned::new())
    }
}
