pub trait Filter {
    type I;

    fn filter(&self, i: &Self::I) -> bool;
}
