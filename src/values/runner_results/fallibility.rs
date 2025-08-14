use std::marker::PhantomData;

pub trait Fallibility {
    type Error;
}

pub struct Infallible;

impl Fallibility for Infallible {
    type Error = ();
}

pub struct Fallible<E>(PhantomData<E>);

impl<E> Fallibility for Fallible<E> {
    type Error = E;
}
