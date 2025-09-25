use crate::Params;
use crate::executor::thread_compute as th;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Fallibility;
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf, ThreadRunnerOf};
use orx_concurrent_iter::ConcurrentIter;

pub fn m<C, I, O, M1, Red>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
    reduce: Red,
) -> (NumSpawned, Option<O>)
where
    C: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
    Red: Fn(O, O) -> O + Sync,
    O: Send,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        Ok(th::reduce::m(thread_runner, iter, state, &map1, &reduce))
    };
    let (num_spawned, result) =
        orchestrator.map_infallible(params, iter, ComputationKind::Collect, thread_map);

    let acc = match result {
        Ok(results) => results.into_iter().flatten().reduce(reduce),
    };

    (num_spawned, acc)
}

type ResultReduce<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn x<C, I, Vo, X1, Red>(
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
    reduce: Red,
) -> (NumSpawned, ResultReduce<Vo>)
where
    C: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(I::Item) -> Vo + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Sync,
{
    let thread_map = |_, iter: &I, state: &SharedStateOf<C>, thread_runner: ThreadRunnerOf<C>| {
        th::reduce::x(thread_runner, iter, state, &xap1, &reduce).into_result()
    };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );
    let acc = result.map(|results| results.into_iter().flatten().reduce(reduce));
    (num_spawned, acc)
}
