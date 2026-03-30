use crate::infallible::{fun::map::FnCopied, xap::Xap};

pub trait XapCopied<'a, O: 'a + Copy>: Xap<O = &'a O> {
    type Copied: Xap<I = Self::I, O = O>;

    fn copied(self) -> Self::Copied;
}

impl<'a, O: 'a + Copy, X: Xap<O = &'a O>> XapCopied<'a, O> for X {
    type Copied = X::Mapped<FnCopied<'a, O>>;

    fn copied(self) -> Self::Copied {
        self.mapped(FnCopied::new())
    }
}
