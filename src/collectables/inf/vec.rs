use crate::collectables::inf::ColIntoInf;
use crate::collectables::utils::{merge_arb_into_first_vec, merge_arb_into_vec, merge_ord_into};
use crate::infallible::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::results::ValsAndIdx;
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ColIntoInf<T> for Vec<T> {
    fn inf_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect(params, iter, x);
        let len: usize = results.iter().map(|x| x.len()).sum();

        let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
        dst.reserve(len);
        merge_ord_into(results, FixedVec::from(dst)).into()
    }

    fn inf_arb_col_into<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_arb(params, iter, x);

        match dst {
            Some(dst) => merge_arb_into_vec(results, dst),
            None => merge_arb_into_first_vec(results),
        }
    }

    fn inf_col_into_new<I, X, R>(dst: Option<Self>, par: Par<I, X, R>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        let results = exe.collect_new(params, iter, x);
        merge_ord_into_new(results, dst)

        // let len: usize = results.iter().map(|x| x.values.len()).sum();

        // let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(len));
        // dst.reserve(len);

        // todo!()
    }
}

fn merge_ord_into_new<T>(mut results: Vec<ValsAndIdx<T>>, dst: Option<Vec<T>>) -> Vec<T> {
    use alloc::vec;
    use orx_priority_queue::{BinaryHeap, PriorityQueue};

    #[derive(Clone)]
    struct VecPos {
        v: usize,
        beg: usize,
        len: usize,
    }

    impl VecPos {
        #[inline(always)]
        fn new(v: usize, beg: usize, len: usize) -> Self {
            Self { v, beg, len }
        }
    }

    let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
    let mut dst = dst.unwrap_or_else(|| Vec::with_capacity(collected_len));
    dst.reserve(collected_len);
    let initial_len = dst.len();
    let total_len = initial_len + collected_len;

    if results.len() == 1 {
        let results = results.into_iter().next().expect("results.len()==1");
        return results.values;
    }

    let mut queue = BinaryHeap::with_capacity(results.len());
    let mut pos_indices = vec![0; results.len()];

    for (v, vec) in results.iter().enumerate() {
        if let Some(pos) = vec.positions.get(0) {
            queue.push(VecPos::new(v, 0, pos.len), pos.idx);
        }
    }
    let mut curr_v = queue.pop_node();
    let mut ptr_dst = dst.as_mut_ptr();

    while let Some(VecPos { v, beg, len }) = curr_v {
        let ptr_src = unsafe { results[v].values.as_ptr().add(beg) };
        unsafe { ptr_dst.copy_from_nonoverlapping(ptr_src, len) };

        pos_indices[v] += 1;
        curr_v = match results[v].positions.get(pos_indices[v]) {
            Some(pos) => {
                let beg = beg + len;
                Some(queue.push_then_pop(VecPos::new(v, beg, pos.len), pos.idx).0)
            }
            None => queue.pop_node(),
        };

        ptr_dst = unsafe { ptr_dst.add(len) };
    }

    for vec in results.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.values.set_len(0) };
    }

    unsafe { dst.set_len(total_len) };

    dst
}
