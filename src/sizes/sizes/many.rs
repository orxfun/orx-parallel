use crate::sizes::Size;

#[derive(Clone, Copy, Default)]
pub struct Many;

impl Size for Many {
    type ThenBin = Many;

    fn size() -> Option<usize> {
        None
    }
}
