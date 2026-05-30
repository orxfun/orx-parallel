use orx_self_or::SoM;

pub trait Using: Sync {
    type Item;

    type ItemKind<'a>: SoM<Self::Item>
    where
        Self: 'a;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn get(&self, thread_idx: usize) -> Self::ItemKind<'_>;
}
