use crate::collectables::Collectable;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Marks the starting input index and number of consecutive elements.
pub struct IdxLen {
    pub idx: usize,
    pub len: usize,
}

/// Collected values and associated indices and lengths of the collected elements.
pub struct ValsAndIdx<T, D = Vec<T>>
where
    T: Send,
    D: Collectable<T>,
{
    pub values: D,
    pub positions: Vec<IdxLen>,
    p: PhantomData<T>,
}

impl<T, D> ValsAndIdx<T, D>
where
    T: Send,
    D: Collectable<T>,
{
    pub fn new() -> Self {
        Self {
            values: D::col_empty(),
            positions: Vec::new(),
            p: PhantomData,
        }
    }

    pub fn new_seq(values: D) -> Self {
        let positions = vec![IdxLen {
            idx: 0,
            len: values.col_len(),
        }];
        Self {
            values,
            positions,
            p: PhantomData,
        }
    }

    #[inline]
    pub fn extend(&mut self, idx: usize, values: impl IntoIterator<Item = T>) {
        let len_begin = self.values.col_len();
        self.values.col_extend(values);

        let len = self.values.col_len() - len_begin;
        self.positions.push(IdxLen { idx, len });
    }

    /// Returns the first observed error if any; returns None if all succeeds.
    #[inline]
    pub fn extend_res<E>(
        &mut self,
        idx: usize,
        values: impl IntoIterator<Item = Result<T, E>>,
    ) -> Option<E> {
        let len_begin = self.values.col_len();
        for x in values {
            match x {
                Ok(x) => self.values.col_push(x),
                Err(e) => {
                    let len = self.values.col_len() - len_begin;
                    self.positions.push(IdxLen { idx, len });
                    return Some(e);
                }
            }
        }

        let len = self.values.col_len() - len_begin;
        self.positions.push(IdxLen { idx, len });

        None
    }

    /// Returns `true` if at least one element is None; returns `false` if all are Some variant.
    #[inline]
    pub fn extend_opt(&mut self, idx: usize, values: impl IntoIterator<Item = Option<T>>) -> bool {
        let len_begin = self.values.col_len();
        for x in values {
            match x {
                Some(x) => self.values.col_push(x),
                None => {
                    let len = self.values.col_len() - len_begin;
                    self.positions.push(IdxLen { idx, len });
                    return true;
                }
            }
        }

        let len = self.values.col_len() - len_begin;
        self.positions.push(IdxLen { idx, len });

        false
    }
}
