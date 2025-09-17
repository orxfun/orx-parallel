use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, NextSuccess, NextWithIdx};
use crate::orch::{NumSpawned, Orchestrator, SharedStateOf};
use crate::runner::{ComputationKind, thread_runner_compute as th};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
) -> (NumSpawned, Option<O>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, thread_runner| {
        Ok(th::next::m(thread_runner, iter, state, &map1))
    };
    let (num_spawned, result) =
        orchestrator.map_infallible(params, iter, ComputationKind::Collect, thread_map);

    let next = match result {
        Ok(results) => results
            .into_iter()
            .filter_map(|x| x)
            .min_by_key(|x| x.0)
            .map(|x| x.1),
    };
    (num_spawned, next)
}

type ResultNext<Vo> = Result<
    Option<(usize, <Vo as Values>::Item)>,
    <<Vo as Values>::Fallibility as Fallibility>::Error,
>;

pub fn x<C, I, Vo, X1>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (NumSpawned, ResultNext<Vo>)
where
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, th_runner| match th::next::x(
        th_runner, iter, state, &xap1,
    ) {
        NextWithIdx::Found { idx, value } => Ok(Some(NextSuccess::Found { idx, value })),
        NextWithIdx::NotFound => Ok(None),
        NextWithIdx::StoppedByWhileCondition { idx } => {
            Ok(Some(NextSuccess::StoppedByWhileCondition { idx }))
        }
        NextWithIdx::StoppedByError { error } => Err(error),
    };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );
    let next = result.map(|results| NextSuccess::reduce(results.into_iter().filter_map(|x| x)));
    (num_spawned, next)
}
