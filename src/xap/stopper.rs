pub trait Stopper {
    type Elem<T>;
}

pub enum NeverStop {}
impl Stopper for NeverStop {
    type Elem<T> = T;
}

pub enum MightStop<E> {
    StoppedByWhilst,
    StoppedByError(E),
}
impl<E> Stopper for MightStop<E> {
    type Elem<T> = Result<T, Self>;
}
