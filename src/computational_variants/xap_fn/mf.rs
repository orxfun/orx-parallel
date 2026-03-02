use crate::computational_variants::xap_fn::{
    filter::Filter, map::Map, map_filter::MapFilter, mfmf::MFMF,
};
use core::marker::PhantomData;

pub struct MF<I, O1, M1, F1>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
{
    m: M1,
    f: F1,
    p: PhantomData<(I, O1)>,
}

impl<I, O1, M1, F1> MF<I, O1, M1, F1>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
{
    pub fn new(m: M1, f: F1) -> Self {
        let p = PhantomData;
        Self { m, f, p }
    }
}

// impl<I, O1, M1, F1> MapFilter<I, O1> for MF<I, O1, M1, F1>
// where
//     M1: Map<I, O1>,
//     F1: Filter<O1>,
// {
//     fn map_filter(&self, i: I) -> Option<O1> {
//         let x = self.m.map(i);
//         match self.f.filter(&x) {
//             true => Some(x),
//             false => None,
//         }
//     }

//     type Compose<Q, M3, F3>
//         = MFMF<I, O1, M1, F1, Q, MF<O1, Q, M3, F3>>
//     where
//         M3: Map<O1, Q>,
//         F3: Filter<Q>;

//     fn compose<Q, M3, F3>(self, m: M3, f: F3) -> Self::Compose<Q, M3, F3>
//     where
//         M3: Map<O1, Q>,
//         F3: Filter<Q>,
//     {
//         MFMF::new(self.m, self.f, MF::new(m, f))
//     }

//     type ComposeF<F3>
//         = MF<I, O1, M1, F1::Compose<F3>>
//     where
//         F3: Filter<O1>;

//     fn compose_f<F3>(self, f: F3) -> Self::ComposeF<F3>
//     where
//         F3: Filter<O1>,
//     {
//         todo!()
//     }
// }

// impl<I, O1, M1, F1> MapFilter<I, O1> for MF<I, O1, M1, F1>
// where
//     M1: Map<I, O1>,
//     F1: Filter<I>,
// {
//     fn map_filter(&self, i: I) -> Option<O1> {
//         todo!()
//     }

//     type Compose<Q, X3, Y3>
//     where
//         X3: Map<O1, Q>,
//         Y3: Filter<Q>;

//     fn compose<Q, X3, Y3>(self, m: X3, f: Y3) -> Self::Compose<Q, X3, Y3>
//     where
//         X3: Map<O1, Q>,
//         Y3: Filter<Q>,
//     {
//         todo!()
//     }
// }

// use crate::computational_variants::xap_fn::{map_filter::MapFilter, mfmf::MFMF};
// use core::marker::PhantomData;

// pub struct MF<I, O, X, Y>
// where
//     X: Fn(I) -> O,
//     Y: Fn(&O) -> bool,
// {
//     m: X,
//     f: Y,
//     p: PhantomData<I>,
// }

// impl<I, O, X, Y> MF<I, O, X, Y>
// where
//     X: Fn(I) -> O,
//     Y: Fn(&O) -> bool,
// {
//     pub fn new(m: X, f: Y) -> Self {
//         let p = PhantomData;
//         Self { m, f, p }
//     }
// }

// // impl<I, O, X, Y> MapFilter<I, O> for MF<I, O, X, Y>
// // where
// //     X: Fn(I) -> O,
// //     Y: Fn(&O) -> bool,
// // {
// //     fn map_filter(&self, i: I) -> Option<O> {
// //         todo!()
// //     }

// //     // type Compose<O3, X3, Y3>
// //     //     = MFMF<I, O, X, Y, O3, MF<O, O3, X3, Y3>>
// //     // where
// //     //     X3: Fn(O) -> O3,
// //     //     Y3: Fn(&O3) -> bool;
// //     // fn compose<O3, X3, Y3>(self, m: X3, f: Y3) -> Self::Compose<O3, X3, Y3>
// //     // where
// //     //     X3: Fn(O) -> O3,
// //     //     Y3: Fn(&O3) -> bool,
// //     // {
// //     //     MFMF::new(self.m, self.f, MF::new(m, f))
// //     // }
// // }
