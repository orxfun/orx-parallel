pub trait Size {
    type ThenBin: Size;
}

pub struct One;

impl Size for One {
    type ThenBin = Bin;
}

pub struct Bin;

impl Size for Bin {
    type ThenBin = Bin;
}

pub struct Many;

impl Size for Many {
    type ThenBin = Many;
}
