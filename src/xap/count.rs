pub trait Count {
    type ThenZeroOne: Count;

    type ThenOne: Count;

    type ThenMany: Count;
}

pub struct ZeroOne;
impl Count for ZeroOne {
    type ThenZeroOne = ZeroOne;

    type ThenOne = ZeroOne;

    type ThenMany = Many;
}

pub struct One;
impl Count for One {
    type ThenZeroOne = ZeroOne;

    type ThenOne = One;

    type ThenMany = Many;
}

pub struct Many;
impl Count for Many {
    type ThenZeroOne = Many;

    type ThenOne = Many;

    type ThenMany = Many;
}
