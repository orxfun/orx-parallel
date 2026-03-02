use crate::computational_variants::xap_fn::{filter::Filter, map::Map, map_filter::MapFilter};
use core::marker::PhantomData;

pub struct MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    m: M1,
    f: F1,
    mf2: Mf2,
    p: PhantomData<(I, O1, O2)>,
}

impl<I, O1, M1, F1, O2, Mf2> MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    pub fn new(m: M1, f: F1, mf2: Mf2) -> Self {
        let p = PhantomData;
        Self { m, f, mf2, p }
    }
}

impl<I, O1, M1, F1, O2, Mf2> MapFilter<I, O2> for MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    fn map_filter(&self, i: I) -> Option<O2> {
        todo!()
    }

    type Compose<Q, M3, F3>
        = MFMF<I, O1, M1, F1, Q, Mf2::Compose<Q, M3, F3>>
    where
        M3: Map<O2, Q>,
        F3: Filter<Q>;

    fn compose<Q, M3, F3>(self, m: M3, f: F3) -> Self::Compose<Q, M3, F3>
    where
        M3: Map<O2, Q>,
        F3: Filter<Q>,
    {
        MFMF::new(self.m, self.f, self.mf2.compose(m, f))
    }
}

// use crate::{
//     computational_variants::xap_fn::{f0::F0, map_filter::MapFilter, xap::XapFn},
//     generic_values::Values,
// };
// use core::marker::PhantomData;

// pub struct MFMF<I, O1, X1, Y1, O2, Mf2>
// where
//     X1: Fn(I) -> O1,
//     Y1: Fn(&O1) -> bool,
//     Mf2: MapFilter<O1, O2>,
// {
//     m1: X1,
//     f1: Y1,
//     mf2: Mf2,
//     p: PhantomData<(I, O2)>,
// }

// impl<I, O1, X1, Y1, O2, Mf2> MFMF<I, O1, X1, Y1, O2, Mf2>
// where
//     X1: Fn(I) -> O1,
//     Y1: Fn(&O1) -> bool,
//     Mf2: MapFilter<O1, O2>,
// {
//     pub fn new(m1: X1, f1: Y1, mf2: Mf2) -> Self {
//         let p = PhantomData;
//         Self { m1, f1, mf2, p }
//     }
// }

// impl<I, O1, X1, Y1, O2, Mf2> MapFilter<I, O2> for MFMF<I, O1, X1, Y1, O2, Mf2>
// where
//     X1: Fn(I) -> O1,
//     Y1: Fn(&O1) -> bool,
//     Mf2: MapFilter<O1, O2>,
// {
//     #[inline(always)]
//     fn map_filter(&self, i: I) -> Option<O2> {
//         let x1 = (self.m1)(i);
//         match (self.f1)(&x1) {
//             true => self.mf2.map_filter(x1),
//             false => None,
//         }
//     }

//     // type Compose<O3, X3, Y3>
//     //     = MFMF<I, O1, X1, Y1, O3, Mf2::Compose<O3, X3, Y3>>
//     // where
//     //     X3: Fn(O2) -> O3,
//     //     Y3: Fn(&O3) -> bool;
//     // fn compose<O3, X3, Y3>(self, m: X3, f: Y3) -> Self::Compose<O3, X3, Y3>
//     // where
//     //     X3: Fn(O2) -> O3,
//     //     Y3: Fn(&O3) -> bool,
//     // {
//     //     MFMF::new(self.m1, self.f1, self.mf2.compose(m, f))
//     // }
// }

// // impl<I, O1, X1, Y1, O2, Mf2> XapFn<I, Option<O2>> for MFMF<I, O1, X1, Y1, O2, Mf2>
// // where
// //     X1: Fn(I) -> O1,
// //     Y1: Fn(&O1) -> bool,
// //     Mf2: MapFilter<O1, O2>,
// // {
// //     fn xap(&self, i: I) -> Option<O2> {
// //         todo!()
// //     }

// //     type Map<Y, Q>
// //     where
// //         Y: Fn(O2) -> Q;

// //     fn map<Y, Q>(self, map: Y) -> Self::Map<Y, Q>
// //     where
// //         Y: Fn(O2) -> Q,
// //     {
// //         let x = F0::<Q>::new();
// //         let x = self.compose(map, F0::<Q>::new());
// //         todo!()
// //     }

// //     type Filter<Y>
// //         = Self
// //     where
// //         Y: Fn(&O2) -> bool;

// //     fn filter<Y>(self, filter: Y) -> Self::Filter<Y>
// //     where
// //         Y: Fn(&O2) -> bool,
// //     {
// //         todo!()
// //     }
// // }
