pub(crate) mod collect_arbitrary;
pub(crate) mod collect_ordered;
pub mod compute;
pub(crate) mod next;
pub(crate) mod next_any;
pub(crate) mod reduce;
pub(crate) mod u_collect_arbitrary;
pub(crate) mod u_collect_ordered;
pub(crate) mod u_next;
pub(crate) mod u_next_any;
pub(crate) mod u_reduce;

pub use compute::ParallelRunnerCompute;
