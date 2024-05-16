use crate::par::Par;
use orx_concurrent_iter::{ConcurrentIter, ExactSizeConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Doubling, Linear, Recursive, SplitVec};
use std::marker::PhantomData;

pub struct ParMap<Data, Out, Map>
where
    Data: ConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
{
    par: Par<Data>,
    map: Map,
    phantom: PhantomData<Out>,
}

impl<Data, Out, Map> ParMap<Data, Out, Map>
where
    Data: ConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
{
    pub(crate) fn new(par: Par<Data>, map: Map) -> Self {
        Self {
            par,
            map,
            phantom: Default::default(),
        }
    }

    #[inline]
    fn map_into<P: PinnedVec<Out>>(self, out: ConcurrentOrderedBag<Out, P>) -> P {
        map_collect(self.par, self.map, out)
    }

    // api

    pub fn to_split_vec(self) -> SplitVec<Out> {
        self.map_into(ConcurrentOrderedBag::with_doubling_growth())
    }

    pub fn to_doubling_split_vec(self) -> SplitVec<Out, Doubling> {
        self.map_into(ConcurrentOrderedBag::with_doubling_growth())
    }

    pub fn to_recursive_split_vec(self) -> SplitVec<Out, Recursive> {
        self.map_into(ConcurrentOrderedBag::with_recursive_growth())
    }

    pub fn to_linear_split_vec(
        self,
        constant_fragment_capacity_exponent: usize,
        fragments_capacity: usize,
    ) -> SplitVec<Out, Linear> {
        self.map_into(ConcurrentOrderedBag::with_linear_growth(
            constant_fragment_capacity_exponent,
            fragments_capacity,
        ))
    }
}

impl<Data, Out, Map> ParMap<Data, Out, Map>
where
    Data: ConcurrentIter + ExactSizeConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
{
    pub fn to_fixed_vec(self) -> FixedVec<Out> {
        let len = self.par.exact_len();
        self.map_into(ConcurrentOrderedBag::with_fixed_capacity(len))
    }

    pub fn to_vec(self) -> Vec<Out> {
        self.to_fixed_vec().into()
    }
}

fn map_collect<Data, Out, Map, P>(par: Par<Data>, map: Map, out: ConcurrentOrderedBag<Out, P>) -> P
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
