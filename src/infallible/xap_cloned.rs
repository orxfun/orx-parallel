use crate::infallible::xap::Xap;

pub trait XapCloned<'a, O: 'a + Clone>: Xap<O = &'a O> {
    type Cloned: Xap<I = Self::I, O = O>;

    fn cloned(self) -> Self::Cloned;
}
