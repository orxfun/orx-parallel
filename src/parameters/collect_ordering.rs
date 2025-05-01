/// Ordering which is important when `collect` is used to consume the
/// parallel iterator to collect the results into a collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CollectOrdering {
    /// The outputs can be, but not necessarily, collected in arbitrary order.
    Arbitrary,
    /// ***Default ordering***.
    ///
    /// The outputs will be collected consistent with the input order.
    #[default]
    Ordered,
}
