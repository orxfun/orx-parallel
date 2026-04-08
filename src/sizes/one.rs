use crate::sizes::{Bin, Size};

pub struct One;

impl Size for One {
    type ThenBin = Bin;
}
