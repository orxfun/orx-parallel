pub trait MapRes: Copy + Send {
    type I;

    type O;

    type E;

    fn map_res(&self, i: Self::I) -> Result<Self::O, Self::E>;
}
