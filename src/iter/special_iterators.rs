use crate::{computational_variants::Par, runner::DefaultOrchestrator};
use orx_concurrent_iter::implementations::ConIterEmpty;

/// An empty parallel iterator which does not yield any elements.
pub type ParEmpty<T, R = DefaultOrchestrator> = Par<ConIterEmpty<T>, R>;

/// Creates an empty parallel iterator which does not yield any elements.
pub fn empty<T: Send + Sync>() -> ParEmpty<T> {
    ParEmpty::new(Default::default(), Default::default(), Default::default())
}
