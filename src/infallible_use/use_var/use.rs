use orx_self_or::SoM;

pub trait Use: Sync {
    type Item;

    type ItemBorrow<'i>: SoM<Self::Item>
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_>;

    fn get_mut(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_>;
}

impl<'a, U: Use> Use for &'a mut U {
    type Item = U::Item;

    type ItemBorrow<'i>
        = U::ItemBorrow<'i>
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item {
        todo!()
    }

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        <U as Use>::get(self, thread_idx)
    }

    fn get_mut(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        <U as Use>::get_mut(self, thread_idx)
    }
}
