pub trait Stopper {}

pub enum NeverStop {}
impl Stopper for NeverStop {}

pub enum MightStop<E> {
    StoppedByWhilst,
    StoppedByError(E),
}
impl<E> Stopper for MightStop<E> {}
