use orx_self_or::SoM;

pub trait Use: Sync {
    type Item;

    type ItemBorrow<'i>: SoM<Self::Item>
    where
        Self: 'i;

    fn init_get(&self, thread_idx: usize) -> Self::ItemBorrow<'_>;

    fn get(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_>;

    fn max_threads(&self) -> Option<usize>;
}
