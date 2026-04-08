use crate::sizes::Size;

pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;
}
