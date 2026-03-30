use crate::{infallible::fun::FnCopied, result::xap_res::XapRes};

pub trait XapResCopied<'a, O: 'a + Copy>: XapRes<O = &'a O> {
    fn copied(self) -> Self::Mapped<FnCopied<'a, O>>;
}

impl<'a, O: 'a + Copy, X: XapRes<O = &'a O>> XapResCopied<'a, O> for X {
    fn copied(self) -> Self::Mapped<FnCopied<'a, O>> {
        self.mapped(FnCopied::new())
    }
}
