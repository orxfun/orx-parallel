use crate::infallible::{fun::map::FnCloned, xap::Xap};

pub trait XapCloned<'a, O: 'a + Clone>: Xap<O = &'a O> {
    type Cloned: Xap<I = Self::I, O = O>;

    fn cloned(self) -> Self::Cloned;
}

impl<'a, O: 'a + Clone, X: Xap<O = &'a O>> XapCloned<'a, O> for X {
    type Cloned = X::Mapped<FnCloned<'a, O>>;

    fn cloned(self) -> Self::Cloned {
        self.mapped(FnCloned::new())
    }
}
