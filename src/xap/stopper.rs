use core::marker::PhantomData;

pub trait Stopper {
    type Elem<T>;
}

pub enum NeverStop {}
impl Stopper for NeverStop {
    type Elem<T> = T;
}

pub struct MightStop<E>(PhantomData<E>);
impl<E> Stopper for MightStop<E> {
    type Elem<T> = Result<T, StoppedBy<E>>;
}

pub enum StoppedBy<E> {
    ByWhilst,
    ByError(E),
}
