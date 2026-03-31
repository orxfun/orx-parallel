use crate::{
    fun_composition::xap::{Elem, Xap},
    generic_values::TransformableValues,
};

pub struct Mm<X: Xap, O, F: Fn(Elem<X>) -> O> {
    x: X,
    f: F,
}

impl<X: Xap, O, F: Fn(Elem<X>) -> O> Mm<X, O, F> {
    pub fn new(x: X, f: F) -> Self {
        Self { x, f }
    }
}
