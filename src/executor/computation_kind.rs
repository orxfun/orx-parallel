/// Computation kind.
#[derive(Clone, Copy)]
pub enum ComputationKind {
    /// Computation where outputs are collected into a collection.
    Collect,
    /// Computation where the inputs or intermediate results are reduced to a single value.
    Reduce,
    /// Computation which allows for early returns, such as `find` operation.
    EarlyReturn,
}
