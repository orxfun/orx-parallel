use crate::generic_values::TransformableValues;

pub trait Xap<I> {
    type O: TransformableValues;

    // type Filter<X>: Xap<I, O = Self::O>
    // where
    //     X: Fn(&Self::O) -> bool;

    // type Map<X, Out>: Xap<I, O = Out>
    // where
    //     X: Fn(Self::O) -> Out;

    fn run(&self, i: I) -> Self::O;
}
