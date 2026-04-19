use crate::sizes::{One, SizePair};

pub trait Size: Clone + Copy + Send + Default {
    type ThenBin: Size;

    type ThenOne: SizePair<S1 = Self, S2 = One>;

    fn elem_len() -> Option<usize>;

    fn output_len(input_len: Option<usize>) -> Option<usize> {
        match (input_len, Self::elem_len()) {
            (Some(input_len), Some(elem_len)) => Some(input_len * elem_len),
            _ => None,
        }
    }
}
