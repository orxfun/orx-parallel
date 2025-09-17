use crate::Params;
use crate::executor::thread_compute as th;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Fallibility;
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
) -> (NumSpawned, Option<O>)
where
    C: ParallelRunner,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, thread_runner| {
        Ok(th::next_any::m(thread_runner, iter, state, &map1))
    };
    let (num_spawned, result) =
        orchestrator.map_infallible(params, iter, ComputationKind::Collect, thread_map);

    let next = match result {
        Ok(results) => results.into_iter().filter_map(|x| x).next(),
    };
    (num_spawned, next)
}

type ResultNextAny<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn x<C, I, Vo, X1>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (NumSpawned, ResultNextAny<Vo>)
where
    C: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, th_runner| {
        th::next_any::x(th_runner, iter, state, &xap1)
    };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );
    let next = result.map(|results| results.into_iter().filter_map(|x| x).next());
    (num_spawned, next)
}
