use crate::out::{out_trait::Out, stop::NeverStop};

pub struct Id<I>(I);

impl<I> Id<I> {
    pub const fn new(i: I) -> Self {
        Self(i)
    }
}

impl<I> Out for Id<I> {
    type Elem = I;

    type Stopper = NeverStop;

    type Values = [Result<I, Self::Stopper>; 1];

    #[inline(always)]
    fn values(self) -> Self::Values {
        [Ok(self.0)]
    }
}
