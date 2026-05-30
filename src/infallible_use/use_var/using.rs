use orx_self_or::SoM;

pub trait Using: Sync {
    type Item;

    type ItemKind<'i>: SoM<Self::Item>
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn get(&self, thread_idx: usize) -> Self::ItemKind<'_>;
}
