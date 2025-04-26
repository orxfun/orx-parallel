#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CollectOrdering {
    Arbitrary,
    #[default]
    Ordered,
}
