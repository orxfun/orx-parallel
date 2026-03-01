use crate::{
    computational_variants::xap_fn::{ff::FF, filter::Filter, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct F<I, X>
where
    X: Fn(&I) -> bool,
{
    f: X,
    p: PhantomData<I>,
}

impl<I, X> Filter<I> for F<I, X>
where
    X: Fn(&I) -> bool,
{
    #[inline(always)]
    fn run(&self, i: &I) -> bool {
        (self.f)(i)
    }

    type Compose<Y>
        = FF<I, X, F<I, Y>>
    where
        Y: Fn(&I) -> bool;
    fn compose<Y: Fn(&I) -> bool>(self, y: Y) -> Self::Compose<Y> {
        FF::new(self.f, F::new(y))
    }
}

impl<I, X> F<I, X>
where
    X: Fn(&I) -> bool,
{
    pub fn new(f: X) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}

// impl<I, F> XapFn<I, Option<I>> for Fil<I, F>
// where
//     F: Fn(&I) -> bool,
// {
//     fn run(&self, i: I) -> Option<I> {
//         todo!()
//     }

//     type Map<X, Q>
//     where
//         X: Fn(<Option<I> as Values>::Item) -> Q;

//     fn map<X, Q>(self, map: X) -> Self::Map<X, Q>
//     where
//         X: Fn(<Option<I> as Values>::Item) -> Q,
//     {
//         todo!()
//     }

//     type Filter<X>
//         = Self
//     where
//         X: Fn(&<Option<I> as Values>::Item) -> bool;

//     fn filter<X>(self, filter: X) -> Self::Filter<X>
//     where
//         X: Fn(&<Option<I> as Values>::Item) -> bool,
//     {
//         todo!()
//     }
// }
