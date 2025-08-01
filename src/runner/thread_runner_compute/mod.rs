#[cfg(test)]
pub(crate) mod collect_arbitrary;
pub(crate) mod collect_ordered;
mod compute;
mod tasks;

pub(crate) use compute::ThreadRunnerCompute;
