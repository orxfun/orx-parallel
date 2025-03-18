#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Collect {
    InArbitraryOrder,
    #[default]
    WithHeapSort,
    WithInPlaceSort,
}
