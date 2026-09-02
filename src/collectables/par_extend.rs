use alloc::vec::Vec;

pub trait ParExtend<T> {
    type ThreadValues;

    type OrderedThreadValues;

    // thread collect

    fn add_thread_value(collected: &mut Self::ThreadValues, value: T);

    fn add_thread_values(collected: &mut Self::ThreadValues, values: impl IntoIterator<Item = T>);

    fn add_ordered_thread_val_and_pos(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        value: T,
    );

    fn add_ordered_thread_values(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = T>,
    );

    // opt: thread collect

    fn add_thread_optionals(
        collected: &mut Self::ThreadValues,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()> {
        for value in values {
            Self::add_thread_value(collected, value?);
        }
        None
    }

    fn add_ordered_thread_optionals(
        collected: &mut Self::OrderedThreadValues,
        idx: usize,
        values: impl IntoIterator<Item = Option<T>>,
    ) -> Option<()> {
        None
    }

    // extend

    fn extend_from_thread_results(&mut self, thread_results: Vec<Self::ThreadValues>);

    fn extend_from_ordered_thread_results(
        &mut self,
        thread_results: Vec<Self::OrderedThreadValues>,
    );
}
