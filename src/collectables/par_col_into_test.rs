use crate::ParCollectInto;
use alloc::vec::Vec;
use core::fmt::Debug;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Doubling, Linear, PseudoDefault, SplitVec};

pub trait ParCollectIntoTest<T: Clone + PartialEq + Debug>:
    ParCollectInto<T> + Clone + PartialEq + Debug + Sized
{
    fn empty() -> Self;

    fn push_back(&mut self, value: T);

    fn init_result(mode: ColIntoMode, val: impl Fn(usize) -> T) -> Option<Self> {
        match mode {
            ColIntoMode::Col => None,
            ColIntoMode::ColIntoEmpty => Some(Self::empty()),
            ColIntoMode::ColIntoFilled(n) => {
                let mut vec = Self::empty();
                for i in 0..n {
                    vec.push_back(val(i));
                }
                Some(vec)
            }
        }
    }

    fn expected(
        mode: ColIntoMode,
        val: impl Fn(usize) -> T,
        iter: impl IntoIterator<Item = T>,
    ) -> Self {
        let mut vec = Self::empty();

        if let ColIntoMode::ColIntoFilled(n) = mode {
            for i in 0..n {
                vec.push_back(val(i));
            }
        }

        for i in iter {
            vec.push_back(i);
        }

        vec
    }
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for FixedVec<T> {
    fn empty() -> Self {
        Self::new(12345)
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for SplitVec<T, Doubling> {
    fn empty() -> Self {
        Self::with_doubling_growth()
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for SplitVec<T, Linear> {
    fn empty() -> Self {
        Self::with_linear_growth(6)
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for Vec<T> {
    fn empty() -> Self {
        PseudoDefault::pseudo_default()
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColIntoMode {
    Col,
    ColIntoEmpty,
    ColIntoFilled(usize),
}
