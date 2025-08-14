use crate::values::runner_results::{
    ArbitraryPush, OrderedPush, SequentialPush, Stop, StopWithIdx,
};
use std::marker::PhantomData;

pub trait Fallibility: Sized {
    type Error: Send;

    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>>;

    fn arbitrary_push_to_stop(arbitrary_push: ArbitraryPush<Self>) -> Option<Stop<Self::Error>>;

    fn sequential_push_to_stop(sequential_push: SequentialPush<Self>) -> Option<Stop<Self::Error>>;
}

pub struct Infallible;

impl Fallibility for Infallible {
    type Error = Never;

    #[inline(always)]
    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>> {
        match ordered_push {
            OrderedPush::StoppedByWhileCondition { idx } => Some(StopWithIdx::DueToWhile { idx }),
            _ => None,
        }
    }

    #[inline(always)]
    fn arbitrary_push_to_stop(arbitrary_push: ArbitraryPush<Self>) -> Option<Stop<Self::Error>> {
        match arbitrary_push {
            ArbitraryPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            _ => None,
        }
    }

    #[inline(always)]
    fn sequential_push_to_stop(sequential_push: SequentialPush<Self>) -> Option<Stop<Self::Error>> {
        match sequential_push {
            SequentialPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            _ => None,
        }
    }
}

pub struct Fallible<E>(PhantomData<E>);

impl<E: Send> Fallibility for Fallible<E> {
    type Error = E;

    #[inline(always)]
    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>> {
        match ordered_push {
            OrderedPush::Done => None,
            OrderedPush::StoppedByWhileCondition { idx } => Some(StopWithIdx::DueToWhile { idx }),
            OrderedPush::StoppedByError { idx, error } => {
                Some(StopWithIdx::DueToError { idx, error })
            }
        }
    }

    #[inline(always)]
    fn arbitrary_push_to_stop(arbitrary_push: ArbitraryPush<Self>) -> Option<Stop<Self::Error>> {
        match arbitrary_push {
            ArbitraryPush::Done => None,
            ArbitraryPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            ArbitraryPush::StoppedByError { error } => Some(Stop::DueToError { error }),
        }
    }

    #[inline(always)]
    fn sequential_push_to_stop(sequential_push: SequentialPush<Self>) -> Option<Stop<Self::Error>> {
        match sequential_push {
            SequentialPush::Done => None,
            SequentialPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            SequentialPush::StoppedByError { error } => Some(Stop::DueToError { error }),
        }
    }
}

pub enum Never {}
