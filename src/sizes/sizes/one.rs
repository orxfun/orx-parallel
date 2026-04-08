use crate::sizes::{Size, sizes::Bin};

pub struct One;

impl Size for One {
    type ThenBin = Bin;
}
