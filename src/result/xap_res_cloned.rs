use crate::{infallible::fun::FnCloned, result::xap_res::XapRes};

pub trait XapResCloned<'a, O: 'a + Clone>: XapRes<O = &'a O> {
    fn cloned(self) -> Self::Mapped<FnCloned<'a, O>>;
}

impl<'a, O: 'a + Clone, X: XapRes<O = &'a O>> XapResCloned<'a, O> for X {
    fn cloned(self) -> Self::Mapped<FnCloned<'a, O>> {
        self.mapped(FnCloned::new())
    }
}
