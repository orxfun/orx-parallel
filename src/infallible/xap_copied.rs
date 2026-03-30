use crate::infallible::{fun::map::FnCopied, xap::Xap};

pub trait XapCopied<'a, O: 'a + Copy>: Xap<O = &'a O> {
    type Copied: Xap<I = Self::I, O = O>;

    fn copied(self) -> Self::Copied;
}

// impl<'a, O: 'a + Copy, X: Xap<O = &'a O>> XapCopied<'a, O> for X {
//     type Copied = X::Map<O, FnCopied<'a, O>>;

//     fn copied(self) -> Self::Copied {
//         todo!()
//     }
// }

// impl<'a, I: 'a + Clone, X: Xap, G: Map<I = X::O, O = &'a I>> XapCloned<'a, I> for M<X, G> {
//     type Cloned = M<Self, FnCloned<'a, I>>;

//     fn cloned(self) -> Self::Cloned {
//         M::new(self, FnCloned::new())
//     }
// }

// impl<'a, I: 'a + Copy, X: Xap, G: Map<I = X::O, O = &'a I>> XapCopied<'a, I> for M<X, G> {
//     type Copied = M<Self, FnCopied<'a, I>>;

//     fn copied(self) -> Self::Copied {
//         M::new(self, FnCopied::new())
//     }
// }
