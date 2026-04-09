use crate::sizes::Size;

#[derive(Clone, Copy, Default)]
pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;
}
