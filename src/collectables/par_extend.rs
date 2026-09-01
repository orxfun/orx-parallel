pub trait ParExtend<T> {
    type ThreadValues;

    type OrderedThreadValues;

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T);

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>);

    fn add_ordered_thread_value(collected: &mut Self::OrderedThreadValues, idx: usize, value: T);

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    );
}

pub trait Contiguous<T> {
    fn reserve(&mut self, additional: usize);

    unsafe fn ptr(&mut self, idx: usize) -> *mut T;

    unsafe fn set_len(&mut self, new_len: usize);
}
