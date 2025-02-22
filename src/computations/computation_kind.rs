#[derive(Clone, Copy)]
pub enum ComputationKind {
    Collect,
    Reduce,
    EarlyReturn,
}
