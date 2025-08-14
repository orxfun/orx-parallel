use std::marker::PhantomData;

pub trait Fallability {
    type Error: Send;
}

pub struct Infallible;

impl Fallability for Infallible {
    type Error = ();
}

pub struct Fallible<E>(PhantomData<E>);

impl<E: Send> Fallability for Fallible<E> {
    type Error = E;
}
