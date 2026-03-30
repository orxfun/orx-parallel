use crate::infallible::size::{One, Size};

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Size: Size;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;
}

// one

pub trait XapOne: Xap<Size = One> {
    #[inline(always)]
    fn one_value(&self, i: Self::I) -> Self::O {
        // SAFETY: by definition the result has exactly one element
        unsafe { self.xap(i).into_iter().next().unwrap_unchecked() }
    }
}

impl<X: Xap<Size = One>> XapOne for X {}
