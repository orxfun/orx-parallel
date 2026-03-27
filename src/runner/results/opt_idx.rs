use crate::runner::results::ValIdx;

pub enum OptIdx<T> {
    Success(ValIdx<T>),
    Fail(usize),
}

impl<T> OptIdx<T> {
    pub fn from_maybe(maybe: Option<T>, idx: usize) -> Self {
        match maybe {
            Some(val) => Self::Success(ValIdx::new(val, idx)),
            None => Self::Fail(idx),
        }
    }
}
