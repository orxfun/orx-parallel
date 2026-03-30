use crate::infallible::Xap;

/// Fallible computation such that:
///
/// * `X1` represents the first part of the computation which exits
///   infallible and enters fallible computation; i.e., `X1: I -> Result<O, E>`.
/// * `X2` represents the second part of the computation which transforms
///   the successful variant of the output `O`, while keeping the error variant
///   as `E`; i.e., `X2: O -> Q`. This allows to work with the success path,
///   while the error results lead to a shortcut in the same manner.
pub struct XapRes<O, E, X1: Xap<O = Result<O, E>>, X2: Xap<I = O>> {
    x1: X1,
    x2: X2,
}

impl<O, E, X1: Xap<O = Result<O, E>>, X2: Xap<I = O>> XapRes<O, E, X1, X2> {
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }

    // compute

    fn xap_res(&self, i: X1::I) {
        todo!()
    }
}
