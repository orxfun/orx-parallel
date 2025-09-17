use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Fallibility;
use crate::orch::{NumSpawned, Orchestrator, SharedStateOf};
use crate::runner::ComputationKind;
use crate::using::runner::thread_runner_compute as th;
use crate::using::using_variants::Using;
use orx_concurrent_iter::ConcurrentIter;

pub fn m<U, C, I, O, M1>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
) -> (NumSpawned, Option<O>)
where
    U: Using,
    C: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    let thread_map = |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner| {
        let u = using.create(nt.into_inner());
        Ok(th::next_any::m(u, thread_runner, iter, state, &map1))
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

pub fn x<U, C, I, Vo, X1>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (NumSpawned, ResultNextAny<Vo>)
where
    U: Using,
    C: Orchestrator,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    let thread_map = |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, th_runner| {
        let u = using.create(nt.into_inner());
        th::next_any::x(u, th_runner, iter, state, &xap1)
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
