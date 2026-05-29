use orx_self_or::SoM;

pub trait Use: Sync {
    type Item;

    type ItemKind: SoM<Self::Item>;

    fn create(&self, thread_idx: usize) -> Self::Item;

    fn get(&self, thread_idx: usize) -> Self::ItemKind;
}
