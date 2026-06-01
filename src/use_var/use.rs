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

// pub struct UsePair<U, V>
// where
//     U: Use,
//     V: Use,
// {
//     u: U,
//     v: V,
// }

// impl<U: Use, V: Use> Use for UsePair<U, V> {
//     type Item = (U::Item, V::Item);

//     type ItemBorrow<'i>
//         = (U::ItemBorrow<'i>, V::ItemBorrow<'i>)
//     where
//         Self: 'i;

//     fn init_get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
//         todo!()
//     }

//     fn get(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
//         todo!()
//     }

//     fn max_threads(&self) -> Option<usize> {
//         todo!()
//     }
// }
