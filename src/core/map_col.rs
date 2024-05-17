use super::run_params::RunParams;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;

pub fn map_col<I, Out, Map, P>(
    params: RunParams,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    match params.is_sequential() {
        true => seq_map_col(iter, map, collected),
        _ => par_map_col(params, iter, map, collected),
    }
}

fn par_map_col<I, Out, Map, P>(
    params: RunParams,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    let offset = collected.len();

    std::thread::scope(|s| {
        let mut num_spawned_threads = 0;
        while params.do_spawn_new(num_spawned_threads, iter.has_more()) {
            num_spawned_threads += 1;

            s.spawn(|| match params.chunk_size() {
                1 => {
                    iter.ids_and_values()
                        .map(|(idx, value)| (offset + idx, map(value)))
                        .for_each(|(idx, value)| unsafe { collected.set_value(idx, value) });
                }
                chunk_size => {
                    let mut buffered = iter.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        let begin_idx = offset + chunk.begin_idx;
                        unsafe { collected.set_values(begin_idx, chunk.values.map(&map)) };
                    }
                }
            });
        }
    });

    unsafe { collected.into_inner().unwrap_only_if_counts_match() }
}

fn seq_map_col<I, Out, Map, P>(iter: I, map: Map, collected: ConcurrentOrderedBag<Out, P>) -> P
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    let mut output = unsafe { collected.into_inner().unwrap_only_if_counts_match() };
    let iter = iter.into_seq_iter();
    for x in iter.map(map) {
        output.push(x);
    }
    output
}
