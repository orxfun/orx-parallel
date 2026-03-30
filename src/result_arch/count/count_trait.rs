pub trait Count {
    // transformations

    type ThenZeroOne: Count;

    type ThenOne: Count;

    type ThenMany: Count;

    // // map

    // type Map<I: IntoIterator, G: Map<I = I::Item>>: IntoIterator<Item = G::O>;

    // fn map<I: IntoIterator, G: Map<I = I::Item>>(i: I, g: G) -> Self::Map<I, G>;

    // // filter_map

    // type FilterMap<I: IntoIterator, G: FilterMap<I = I::Item>>: IntoIterator<Item = G::O>;

    // fn filter_map<I: IntoIterator, G: FilterMap<I = I::Item>>(i: I, g: G) -> Self::FilterMap<I, G>;

    // // flat_map

    // type FlatMap<I: IntoIterator, G: FlatMap<I = I::Item>>: IntoIterator<
    //     Item = <G::O as IntoIterator>::Item,
    // >;

    // fn flat_map<I: IntoIterator, G: FlatMap<I = I::Item>>(i: I, g: G) -> Self::FlatMap<I, G>;
}
