use crate::{par::Par, params};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{AtomicCounter, ConcurrentIter, ExactSizeConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Doubling, Linear, Recursive, SplitVec};

pub struct ParFilter<Data, Filter>
where
    Data: ConcurrentIter,
    Filter: Fn(Data::Item) -> bool + Send + Sync,
{
    par: Par<Data>,
    filter: Filter,
}

impl<Data, Filter> ParFilter<Data, Filter>
where
    Data: ConcurrentIter,
    Filter: Fn(Data::Item) -> bool + Send + Sync,
{
    pub(crate) fn new(par: Par<Data>, filter: Filter) -> Self {
        Self { par, filter }
    }
}

fn filter<Data, Filter, P, Q>(
    par: Par<Data>,
    filter: Filter,
    collected: &ConcurrentBag<Data::Item, P>,
    positions: &ConcurrentOrderedBag<usize, Q>,
) where
    Data: ConcurrentIter,
    Filter: Fn(&Data::Item) -> bool + Send + Sync,
    P: PinnedVec<Data::Item>,
    Q: PinnedVec<usize>,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let filter = &filter;
    let inputs = &par.into_data();

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || match chunk_size {
                1 => {
                    while let Some(x) = inputs.next_id_and_value() {
                        let position = match filter(&x.value) {
                            true => collected.push(x.value),
                            false => usize::MAX,
                        };
                        unsafe { positions.set_value(x.idx, position) };
                    }
                }
                _ => {
                    let mut buffered = inputs.buffered_iter(chunk_size);
                    let mut local = vec![0usize; chunk_size];
                    while let Some(chunk) = buffered.next() {
                        for (i, value) in chunk.values.enumerate() {
                            let position = match filter(&value) {
                                true => collected.push(value),
                                false => usize::MAX,
                            };
                            local[i] = position;
                        }
                        unsafe { positions.set_values(chunk.begin_idx, local.iter().cloned()) };
                    }
                }
            });
        }
    });
}

fn filter_unordered<Data, Filter, P>(
    par: Par<Data>,
    filter: Filter,
    collected: &ConcurrentBag<Data::Item, P>,
) where
    Data: ConcurrentIter,
    Filter: Fn(&Data::Item) -> bool + Send + Sync,
    P: PinnedVec<Data::Item>,
{
    let (num_threads, chunk_size) = par.eval_num_threads_chunk_size();

    let filter = &filter;
    let inputs = &par.into_data();

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || match chunk_size {
                1 => {
                    while let Some(value) = inputs.next() {
                        if filter(&value) {
                            collected.push(value);
                        }
                    }
                }
                _ => {
                    let mut buffered = inputs.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        for value in chunk.values.filter(filter) {
                            collected.push(value);
                        }
                    }
                }
            });
        }
    });
}
