use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Fallibility;
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf};
use crate::using::executor::thread_compute as th;
use crate::using::using_variants::Using;
use orx_concurrent_iter::ConcurrentIter;

pub fn m<'using, U, C, I, O, M1>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    map1: M1,
) -> (NumSpawned, Option<O>)
where
    U: Using<'using>,
    C: ParallelRunner,
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
        Ok(results) => results.into_iter().flatten().next(),
    };
    (num_spawned, next)
}

type ResultNextAny<Vo> =
    Result<Option<<Vo as Values>::Item>, <<Vo as Values>::Fallibility as Fallibility>::Error>;

pub fn x<'using, U, C, I, Vo, X1>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (NumSpawned, ResultNextAny<Vo>)
where
    U: Using<'using>,
    C: ParallelRunner,
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
    let next = result.map(|results| results.into_iter().flatten().next());
    (num_spawned, next)
}
