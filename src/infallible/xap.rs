use crate::infallible::size::Size;

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Size: Size;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;
}
