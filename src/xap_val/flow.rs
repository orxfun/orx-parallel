use core::marker::PhantomData;

pub trait Flow {}

pub struct Cont;
impl Flow for Cont {}

pub struct StopWhile;
impl Flow for StopWhile {}

pub struct StopErr<E>(PhantomData<E>);
impl<E> Flow for StopErr<E> {}

pub struct StopWhileOrErr<E>(PhantomData<E>);
impl<E> Flow for StopWhileOrErr<E> {}

pub trait MustStop {
    fn must_stop(res: &Self) -> bool;
}

impl MustStop for () {
    #[inline(always)]
    fn must_stop(_: &Self) -> bool {
        false
    }
}
