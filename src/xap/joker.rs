use crate::xap::xap_trait::Xap;
use core::marker::PhantomData;

pub struct Joker<I, O>(PhantomData<(I, O)>);

impl<I, O> Xap for Joker<I, O> {
    type I = I;

    type O = O;

    type Map<Q, G>
        = Joker<I, Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>
        = Joker<I, O>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Joker<I, O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Joker<I, Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Joker<I, V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;

    type Res = ();

    type ResOf<T> = T;

    fn push_to_vec_with_idx(
        &self,
        i: Self::I,
        idx: usize,
        vec: &mut std::vec::Vec<(usize, Self::O)>,
    ) -> Self::Res {
        todo!()
    }

    fn push_to_pinned_vec<P: orx_fixed_vec::PinnedVec<Self::O>>(
        &self,
        i: Self::I,
        vector: &mut P,
    ) -> Self::Res {
        todo!()
    }

    fn next_any(&self, i: Self::I) -> Self::ResOf<Option<Self::O>> {
        todo!()
    }

    fn next(&self, i: Self::I) -> Self::ResOf<Option<Self::O>> {
        todo!()
    }

    fn reduce<R>(&self, reduce: R, i: Self::I, acc: Option<Self::O>) -> Self::ResOf<Option<Self::O>>
    where
        R: Fn(Self::O, Self::O) -> Self::O,
    {
        todo!()
    }
}
