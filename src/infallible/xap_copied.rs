use crate::infallible::{fun::FnCopied, xap::Xap};

pub trait XapCopied<'a, O: 'a + Copy>: Xap<O = &'a O> {
    fn copied(self) -> Self::Mapped<FnCopied<'a, O>>;
}

impl<'a, O: 'a + Copy, X: Xap<O = &'a O>> XapCopied<'a, O> for X {
    fn copied(self) -> Self::Mapped<FnCopied<'a, O>> {
        self.mapped(FnCopied::new())
    }
}
