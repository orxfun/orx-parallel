use crate::result::count::Count;

pub trait XapRes: Copy + Send {
    type I;

    type O;

    type E;

    type Count: Count;

    type Values: IntoIterator<Item = Result<Self::O, Self::E>>;

    fn xap(&self, i: Self::I) -> Self::Values;
}
