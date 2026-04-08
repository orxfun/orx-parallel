use crate::sizes::Size;

pub struct Many;

impl Size for Many {
    type ThenBin = Many;
}
