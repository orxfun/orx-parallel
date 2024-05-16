use crate::par::Par;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;

pub fn map_collect<Data, Out, Map, P>(
    par: Par<Data>,
    map: Map,
    out: ConcurrentOrderedBag<Out, P>,
) -> P
where
    Data: ConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
    P: PinnedVec<Out>,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let map = &map;
    let inputs = &par.into_data();
    let outputs = &out;

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || match chunk_size {
                1 => {
                    while let Some(x) = inputs.next_id_and_value() {
                        unsafe { outputs.set_value(x.idx, map(x.value)) };
                    }
                }
                _ => {
                    let mut buffered = inputs.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        let (begin_idx, n) = (chunk.begin_idx, chunk.values.len());
                        unsafe { outputs.set_n_values(begin_idx, n, chunk.values.map(map)) };
                    }
                }
            });
        }
    });

    unsafe { out.into_inner().unwrap_only_if_counts_match() }
}
