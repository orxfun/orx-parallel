#[cfg(test)]
mod collect_arbitrary;
mod collect_ordered;
mod compute;
mod tasks;

pub(crate) use compute::ThreadRunnerCompute;
