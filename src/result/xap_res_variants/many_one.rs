use crate::infallible::{Many, One, Xap};
use crate::result::xap_res::XapRes;
use core::marker::PhantomData;

pub struct XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = Many>,
    X2: Xap<I = M, Count = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = Many>,
    X2: Xap<I = M, Count = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

// impl<M, E, X1, X2> XapRes for XapResManyOne<M, E, X1, X2>
// where
//     X1: Xap<O = Result<M, E>, Count = Many>,
//     X2: Xap<I = M, Count = One>,
// {
//     type M = M;

//     type E = E;

//     type X1 = X1;

//     type X2 = X2;

//     type Values = <X2 as Xap>::Values;

//     #[inline(always)]
//     fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Result<Self::Values, Self::E> {
//         let a = self.x1.xap(i);
//         let b = a
//             .into_iter()
//             .map(|a| a.map(|a| unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() }));

//         //  Result<<X2 as Xap>::O, E>
//         // Map<X1::Values::IntoIter, impl FnMut(Result<M, E>) -> Result<<X2 as Xap>::O, E>>
//         todo!()
//     }
// }

// iter

pub struct IterResManyOne<'a, M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Count = One>,
{
    iter: I,
    x2: &'a X2,
}

impl<'a, M, E, I, X2> Iterator for IterResManyOne<'a, M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Count = One>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|a| a.map(|a| unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() }))
    }
}
