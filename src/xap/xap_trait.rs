use crate::xap::count::Count;

pub type IterOf<X> = <<X as Xap>::Values as IntoIterator>::IntoIter;

pub trait Xap {
    type I;

    type O;

    type Count: Count;

    type Values: IntoIterator<Item = Self::O>;

    fn xap(&self, i: Self::I) -> Self::Values;

    // transformations

    type Map<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Q + Copy;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy;

    type Inspect<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) + Copy;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy;

    type Filter<H>: Xap<I = Self::I, O = Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy;

    type FilterMap<Q, H>: Xap<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy;

    type FlatMap<V, H>: Xap<I = Self::I, O = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy;
}

pub trait XapCloned<'a, O: 'a + Clone>: Xap<O = &'a O> {
    type Cloned: Xap<I = Self::I, O = O>;

    fn cloned(self) -> Self::Cloned;
}

pub trait XapCopied<'a, O: 'a + Copy>: Xap<O = &'a O> {
    type Copied: Xap<I = Self::I, O = O>;

    fn copied(self) -> Self::Copied;
}
