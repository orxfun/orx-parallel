use crate::sizes::{Many, Size};

pub trait SizePair: Clone + Copy + Send + Default {
    type S1: Size;

    type S2: Size;

    type ThenBin: SizePair<S1 = Self::S1, S2 = <Self::S2 as Size>::ThenBin>;

    type ThenMany: SizePair<S1 = Self::S1, S2 = Many>;
}
