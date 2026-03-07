use crate::xap::fun::filter::r#fn::FilterFn;

pub trait FilterQ: FilterFn {
    type Then<H>: FilterQ<I = Self::I>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>;
}
