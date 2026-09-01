use alloc::vec::Vec;

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

    // extend

    fn extend_from_thread_results(&mut self, thread_results: Vec<Self::ThreadValues>);

    fn extend_from_ordered_thread_results(
        &mut self,
        thread_results: Vec<Self::OrderedThreadValues>,
    );
}
