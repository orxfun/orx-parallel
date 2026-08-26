use crate::common_par_traits::ParInfCommon;
use crate::infallible::recursive::par::ParRec;
use crate::infallible::recursive::par_core::ParRecCore;
use crate::infallible::{ParCore, ParIter, Xap};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::pool::ParThreadPool;
use crate::runner::{DefaultRunner, ParRunner};
use crate::{Par, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator.
pub struct ParIterRecursive<I, X, Ix, Ex, R = DefaultRunner>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&X::I) -> Ix + Send + Sync,
{
    iter: I,
    xap: X,
    exe: R,
    params: Params,
    extend: Ex,
}

impl<I, X, Ix, Ex, R> ParIterRecursive<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&X::I) -> Ix + Send + Sync,
{
    pub(crate) fn new(iter: I, xap: X, exe: R, params: Params, extend: Ex) -> Self {
        Self {
            iter,
            xap,
            exe,
            params,
            extend,
        }
    }

    pub(super) fn with_xap<Y: Xap<I = I::Item>>(self, xap: Y) -> ParIterRecursive<I, Y, Ix, Ex, R> {
        ParIterRecursive::new(self.iter, xap, self.exe, self.params, self.extend)
    }

    fn destruct_x(self) -> (I, X, R, Params, Ex) {
        (self.iter, self.xap, self.exe, self.params, self.extend)
    }
}

impl<I, X, Ix, Ex, R> ParRecCore for ParIterRecursive<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&X::I) -> Ix + Send + Sync,
{
    type Item = X::O;

    type Runner = R;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }
}

// impl<I, X, I2, E, R> ParInfCommon for ParIterRecursive<I, X, I2, E, R>
// where
//     I: IntoIterator,
//     X: Xap<I = I::Item>,
//     R: ParRunner,
//     I2: IntoIterator<Item = X::I>,
//     E: Fn(&X::I) -> I2 + Send + Sync,
// {
//     type CommonItem = X::O;

//     fn common_collect_into<C>(self, dst: &mut C)
//     where
//         C: ParCollectInto<Self::CommonItem>,
//         Self::CommonItem: Send,
//     {
//     }

//     // type CommonItem = <Self as ParRecCore>::Item;

//     // fn common_collect_into<C>(self, dst: &mut C)
//     // where
//     //     C: ParCollectInto<Self::CommonItem>,
//     //     Self::CommonItem: Send,
//     // {
//     //     // self.collect_into(dst);
//     // }
// }

// impl<I, X, C, E, R> ParRec for ParIterRecursive<I, X, C, E, R>
// where
//     I: IntoIterator,
//     X: Xap<I = I::Item>,
//     R: ParRunner,
//     C: IntoIterator<Item = X::I>,
//     E: Fn(&X::I) -> C + Send + Sync,
// {
//     fn runner<Q: ParRunner>(
//         self,
//         runner: Q,
//     ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
//         let (iter, xap, _, params, extend) = self.destruct();
//         ParIterRecursive {
//             iter,
//             xap,
//             exe: runner,
//             params,
//             extend,
//         }
//     }

//     fn runner_with_diagnostics(
//         self,
//     ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
//         todo!()
//     }

//     fn pool<P: ParThreadPool>(
//         self,
//         pool: P,
//     ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
//         todo!()
//     }

//     fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
//         todo!()
//     }

//     fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
//         todo!()
//     }

//     fn iteration_order(self, collect: IterationOrder) -> Self {
//         todo!()
//     }

//     fn map<Q, H>(
//         self,
//         h: H,
//     ) -> impl ParRec<Item = Q, Xap = crate::infallible::MapOf<Self::Xap, Q, H>, Input = Self::Input>
//     where
//         H: Fn(Self::Item) -> Q + Copy + Send,
//     {
//         todo!()
//     }

//     fn inspect<H>(
//         self,
//         h: H,
//     ) -> impl ParRec<Item = Self::Item, Xap = crate::infallible::InsOf<Self::Xap, H>, Input = Self::Input>
//     where
//         H: Fn(&Self::Item) + Copy + Send,
//     {
//         todo!()
//     }

//     fn filter<H>(
//         self,
//         h: H,
//     ) -> impl ParRec<Item = Self::Item, Xap = crate::infallible::FilOf<Self::Xap, H>, Input = Self::Input>
//     where
//         H: Fn(&Self::Item) -> bool + Copy + Send,
//     {
//         todo!()
//     }

//     fn filter_map<Q, H>(
//         self,
//         h: H,
//     ) -> impl ParRec<Item = Q, Xap = crate::infallible::FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
//     where
//         H: Fn(Self::Item) -> Option<Q> + Copy + Send,
//     {
//         todo!()
//     }

//     fn flat_map<V, H>(
//         self,
//         h: H,
//     ) -> impl ParRec<
//         Item = V::Item,
//         Xap = crate::infallible::FlatMapOf<Self::Xap, V, H>,
//         Input = Self::Input,
//     >
//     where
//         V: IntoIterator,
//         H: Fn(Self::Item) -> V + Copy + Send,
//     {
//         todo!()
//     }

//     fn flatten(
//         self,
//     ) -> impl ParRec<
//         Item = <Self::Item as IntoIterator>::Item,
//         Xap = crate::infallible::FlattenOf<Self::Xap>,
//         Input = Self::Input,
//     >
//     where
//         Self::Item: IntoIterator,
//     {
//         todo!()
//     }

//     fn first(self) -> Option<Self::Item>
//     where
//         Self::Item: Send,
//     {
//         todo!()
//     }

//     fn reduce<F>(self, f: F) -> Option<Self::Item>
//     where
//         F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
//         Self::Item: Send,
//     {
//         todo!()
//     }

//     fn collect_into<C>(self, dst: &mut C)
//     where
//         C: ParCollectInto<Self::Item>,
//         Self::Item: Send,
//     {
//         todo!()
//     }

//     fn collect<C>(self) -> C
//     where
//         C: ParCollectInto<Self::Item>,
//         Self::Item: Send,
//     {
//         todo!()
//     }
// }
