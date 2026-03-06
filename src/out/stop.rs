pub trait Stopper {}

pub enum NeverStop {}
impl Stopper for NeverStop {}

pub struct StoppedByWhilst {}
impl Stopper for StoppedByWhilst {}

pub struct StoppedByError<E>(E);
impl<E> Stopper for StoppedByError<E> {}

pub enum StoppedByWhilstOrError<E> {
    StoppedByWhilst(StoppedByWhilst),
    StoppedByError(StoppedByError<E>),
}
impl<E> Stopper for StoppedByWhilstOrError<E> {}
