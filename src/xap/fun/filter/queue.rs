use crate::xap::fun::filter::fn_trait::Filter;

pub trait FilterQueue: Filter {
    type Then<H>: FilterQueue<I = Self::I>
    where
        H: Filter<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: Filter<I = Self::I>;
}
