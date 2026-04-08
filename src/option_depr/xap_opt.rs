use crate::infallible::fun::Map;

pub trait XapOpt: Copy + Send {
    // Type of the input elements.
    type I;

    /// Type of the intermediate success value bridging between `X1` and `X2`.
    type M;

    /// Final success value.
    type O;

    type Results: IntoIterator<Item = Option<Self::O>>;

    fn xap_res(&self, i: Self::I) -> Self::Results;

    // transformations

    type Map<Q, H>: XapOpt<I = Self::I, M = Self::M, O = Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    type Inspect<H>: XapOpt<I = Self::I, M = Self::M, O = Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send;

    type Filter<H>: XapOpt<I = Self::I, M = Self::M, O = Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: XapOpt<I = Self::I, M = Self::M, O = Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: XapOpt<I = Self::I, M = Self::M, O = <V as IntoIterator>::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<H>: XapOpt<I = Self::I, M = Self::M, O = H::O>
    where
        H: Map<I = Self::O>;

    fn mapped<M>(self, h: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>;
}
