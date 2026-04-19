use crate::sizes::{ManyOne, Size};

#[derive(Clone, Copy, Default)]
pub struct Many;

impl Size for Many {
    type ThenBin = Many;

    type IntoPair = ManyOne;

    fn elem_len() -> Option<usize> {
        None
    }
}
