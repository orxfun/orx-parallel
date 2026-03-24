use crate::xap::count::Count;

pub trait Xap: Copy + Send {
    type I;

    type O;

    type Count: Count;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;

    fn into_iter_over(
        self,
        inputs: impl IntoIterator<Item = Self::I>,
    ) -> impl Iterator<Item = Self::O>;

    // transformations

    type Map<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    type Inspect<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send;

    type Filter<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;
}

pub trait XapCloned<'a, O: 'a + Clone>: Xap<O = &'a O> {
    type Cloned: Xap<I = Self::I, O = O>;

    fn cloned(self) -> Self::Cloned;
}

pub trait XapCopied<'a, O: 'a + Copy>: Xap<O = &'a O> {
    type Copied: Xap<I = Self::I, O = O>;

    fn copied(self) -> Self::Copied;
}
