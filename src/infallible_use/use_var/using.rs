use orx_self_or::SoM;

pub trait Using: Sync {
    type Item;

    type ItemBorrow<'i>: SoM<Self::Item>
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_>;
}

impl<'a, U: Using> Using for &'a mut U {
    type Item = U::Item;

    type ItemBorrow<'i>
        = U::ItemBorrow<'i>
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item {
        todo!()
    }

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        <U as Using>::get(self, thread_idx)
    }
}
