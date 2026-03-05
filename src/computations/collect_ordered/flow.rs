use crate::xap_val::{Cont, Flow, MustStop, Never, StopErr, StopWhile, StopWhileOrErr};

pub trait FlowCollectOrdered: Flow {
    type Stop: MustStop;
}

impl FlowCollectOrdered for Cont {
    type Stop = ();
}

pub enum StoppedByWhile {
    While(usize),
}

impl MustStop for Option<StoppedByWhile> {
    #[inline(always)]
    fn must_stop(res: &Self) -> bool {
        res.is_some()
    }
}

impl FlowCollectOrdered for StopWhile {
    type Stop = Option<StoppedByWhile>;
}

pub enum StoppedByErr<E> {
    Error(usize, E),
}

impl<E> MustStop for Option<StoppedByErr<E>> {
    #[inline(always)]
    fn must_stop(res: &Self) -> bool {
        res.is_some()
    }
}

impl<E> FlowCollectOrdered for StopErr<E> {
    type Stop = Option<StoppedByErr<E>>;
}

pub enum StoppedByWhileOrErr<E> {
    While(usize),
    Error(usize, E),
}

impl<E> MustStop for Option<StoppedByWhileOrErr<E>> {
    #[inline(always)]
    fn must_stop(res: &Self) -> bool {
        res.is_some()
    }
}

impl<E> FlowCollectOrdered for StopWhileOrErr<E> {
    type Stop = Option<StoppedByWhileOrErr<E>>;
}
