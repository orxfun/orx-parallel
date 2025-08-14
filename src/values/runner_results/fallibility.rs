use crate::values::{
    Values,
    runner_results::{
        OrderedPush, ThreadCollect,
        stop::{Stop, StopWithIdx},
    },
};
use std::marker::PhantomData;

pub trait Fallibility: Sized {
    type Error: Send;

    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>>;
}

pub struct Infallible;

impl Fallibility for Infallible {
    type Error = Never;

    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>> {
        match ordered_push {
            OrderedPush::StoppedByWhileCondition { idx } => Some(StopWithIdx::DueToWhile { idx }),
            _ => None,
        }
    }
}

pub struct Fallible<E>(PhantomData<E>);

impl<E: Send> Fallibility for Fallible<E> {
    type Error = E;

    fn ordered_push_to_stop(ordered_push: OrderedPush<Self>) -> Option<StopWithIdx<Self::Error>> {
        match ordered_push {
            OrderedPush::StoppedByWhileCondition { idx } => Some(StopWithIdx::DueToWhile { idx }),
            OrderedPush::StoppedByError { idx, error } => {
                Some(StopWithIdx::DueToError { idx, error })
            }
            OrderedPush::Done => None,
        }
    }
}

pub enum Never {}
