use crate::values::{
    Values,
    runner_results::{
        ArbitraryPush, OrderedPush, Reduce, SequentialPush, Stop, StopWithIdx, stop::StopReduce,
    },
};
use std::marker::PhantomData;

pub trait Fallibility: Sized {
    type Error: Send;

    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>>;

    fn arbitrary_push_to_stop(arbitrary_push: ArbitraryPush<Self>) -> Option<Stop<Self::Error>>;

    fn sequential_push_to_stop(sequential_push: SequentialPush<Self>) -> Option<Stop<Self::Error>>;

    fn reduce_to_stop<V>(reduce: Reduce<V>) -> Result<Option<V::Item>, StopReduce<V>>
    where
        V: Values<Fallibility = Self>;
}

pub struct Infallible;

impl Fallibility for Infallible {
    type Error = Never;

    #[inline(always)]
    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>> {
        match ordered_push {
            OrderedPush::Done => None,
            OrderedPush::StoppedByWhileCondition { idx } => Some(StopWithIdx::DueToWhile { idx }),
        }
    }

    #[inline(always)]
    fn arbitrary_push_to_stop(arbitrary_push: ArbitraryPush<Self>) -> Option<Stop<Self::Error>> {
        match arbitrary_push {
            ArbitraryPush::Done => None,
            ArbitraryPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
        }
    }

    #[inline(always)]
    fn sequential_push_to_stop(sequential_push: SequentialPush<Self>) -> Option<Stop<Self::Error>> {
        match sequential_push {
            SequentialPush::StoppedByWhileCondition => Some(Stop::DueToWhile),
            _ => None,
        }
    }

    #[inline(always)]
    fn reduce_to_stop<V>(reduce: Reduce<V>) -> Result<Option<V::Item>, StopReduce<V>>
    where
        V: Values<Fallibility = Self>,
    {
        match reduce {
            Reduce::Done { acc } => Ok(acc),
            Reduce::StoppedByWhileCondition { acc } => Err(StopReduce::DueToWhile { acc }),
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

    #[inline(always)]
    fn reduce_to_stop<V>(reduce: Reduce<V>) -> Result<Option<V::Item>, StopReduce<V>>
    where
        V: Values<Fallibility = Self>,
    {
        match reduce {
            Reduce::Done { acc } => Ok(acc),
            Reduce::StoppedByWhileCondition { acc } => Err(StopReduce::DueToWhile { acc }),
            Reduce::StoppedByError { error } => Err(StopReduce::DueToError { error }),
        }
    }
}

pub enum Never {}
