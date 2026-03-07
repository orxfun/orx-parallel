use crate::xap::fun::filter::r#fn::FilterFn;

pub trait FilterQueue: FilterFn {
    type Then<H>: FilterQueue<I = Self::I>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>;
}
