use crate::Params;
use crate::generic_values::Values;
use crate::generic_values::runner_results::{Fallibility, NextSuccess, NextWithIdx};
use crate::runner::{ComputationKind, NumSpawned, ParallelRunner, SharedStateOf};
use crate::using::executor::thread_compute as th;
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
    C: ParallelRunner,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    let thread_map = |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, thread_runner| {
        let u = using.create(nt.into_inner());
        Ok(th::next::m(u, thread_runner, iter, state, &map1))
    };
    let (num_spawned, result) =
        orchestrator.map_infallible(params, iter, ComputationKind::Collect, thread_map);

    let next = match result {
        Ok(results) => results
            .into_iter()
            .flatten()
            .min_by_key(|x| x.0)
            .map(|x| x.1),
    };
    (num_spawned, next)
}

type ResultNext<Vo> = Result<
    Option<(usize, <Vo as Values>::Item)>,
    <<Vo as Values>::Fallibility as Fallibility>::Error,
>;

pub fn x<U, C, I, Vo, X1>(
    using: U,
    mut orchestrator: C,
    params: Params,
    iter: I,
    xap1: X1,
) -> (NumSpawned, ResultNext<Vo>)
where
    U: Using,
    C: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
{
    let thread_map = |nt: NumSpawned, iter: &I, state: &SharedStateOf<C>, th_runner| {
        let u = using.create(nt.into_inner());
        match th::next::x(u, th_runner, iter, state, &xap1) {
            NextWithIdx::Found { idx, value } => Ok(Some(NextSuccess::Found { idx, value })),
            NextWithIdx::NotFound => Ok(None),
            NextWithIdx::StoppedByWhileCondition { idx } => {
                Ok(Some(NextSuccess::StoppedByWhileCondition { idx }))
            }
            NextWithIdx::StoppedByError { error } => Err(error),
        }
    };
    let (num_spawned, result) = orchestrator.map_all::<Vo::Fallibility, _, _, _>(
        params,
        iter,
        ComputationKind::Collect,
        thread_map,
    );
    let next = result.map(|results| NextSuccess::reduce(results.into_iter().flatten()));
    (num_spawned, next)
}
