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
    fn filter(&self, i: &I) -> bool {
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

// impl<I, X> F<I, X>
// where
//     X: Fn(&I) -> bool,
// {
//     pub fn new(f: X) -> Self {
//         let p = PhantomData;
//         Self { f, p }
//     }
// }

// impl<I, X> XapFn<I, Option<I>> for F<I, X>
// where
//     X: Fn(&I) -> bool,
// {
//     fn run(&self, i: I) -> Option<I> {
//         match self.filter(&i) {
//             true => Some(i),
//             false => None,
//         }
//     }

//     type Map<Y, Q>
//     where
//         Y: Fn(I) -> Q;

//     fn map<Y, Q>(self, map: Y) -> Self::Map<Y, Q>
//     where
//         Y: Fn(I) -> Q,
//     {
//         todo!()
//     }

//     type Filter<Y>
//         = FF<I, X, F<I, Y>>
//     where
//         Y: Fn(&I) -> bool;

//     fn filter<Y>(self, filter: Y) -> Self::Filter<Y>
//     where
//         Y: Fn(&I) -> bool,
//     {
//         self.compose(filter)
//     }
// }
