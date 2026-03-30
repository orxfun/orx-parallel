use crate::result_arch::count::{Count, Many, ZeroOne};
// use crate::result::fun::{filter_map::FilterMap, flat_map::FlatMap, map::Map};

pub struct One;

impl Count for One {
    // transformations

    type ThenZeroOne = ZeroOne;

    type ThenOne = One;

    type ThenMany = Many;

    // // map

    // type Map<I: IntoIterator, G: Map<I = I::Item>> = [G::O; 1];

    // #[inline(always)]
    // fn map<I: IntoIterator, G: Map<I = I::Item>>(i: I, g: G) -> Self::Map<I, G> {
    //     let x = unsafe { i.into_iter().next().unwrap_unchecked() };
    //     [g.map(x)]
    // }

    // // filter_map

    // type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>> = Option<G::O>;

    // #[inline(always)]
    // fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(i: I, g: G) -> Self::FilterMap<I, G> {
    //     let x = unsafe { i.into_iter().next().unwrap_unchecked() };
    //     g.filter_map(x)
    // }

    // // flat_map

    // type FlatMap<I: IntoIterator, G: FlatMap<I = I::Item>> = G::O;

    // #[inline(always)]
    // fn flat_map<I: IntoIterator, G: FlatMap<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G> {
    //     let x = unsafe { i.into_iter().next().unwrap_unchecked() };
    //     g.flat_map(x)
    // }
}
