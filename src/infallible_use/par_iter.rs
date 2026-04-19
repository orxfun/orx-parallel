use crate::{infallible_use::Use, runner::ParRunner};

pub trait ParUseIter {
    type Runner: ParRunner;

    type Item;

    type Use: Use;
}
