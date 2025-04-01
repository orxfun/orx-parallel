/// A type that distinguishes whether or not a folded over value is still the initial
/// identity or it is a value computed by composing the identity with at least one
/// element.
///
/// Although irrelevant for sequential programs, distinguishing the two variants is
/// crucial in a parallel program not to double-count, double-fold the identity.
/// In other words, it makes sure that the result of a parallel-fold is independent
/// of the number of threads used to compute the result.
pub enum FoldResult<T> {
    Identity(T),
    Aggregate(T),
}
