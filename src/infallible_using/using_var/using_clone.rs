use crate::infallible_using::using_var::using::Using;

pub struct UsingClone<T: Clone + Send>(T);

impl<T: Clone + Send> Using for UsingClone<T> {
    type Item = T;

    #[inline]
    fn create(&self, _: usize) -> Self::Item {
        self.0.clone()
    }

    #[inline]
    fn into_inner(self) -> Self::Item {
        self.0
    }
}
