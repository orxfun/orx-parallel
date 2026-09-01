pub trait ParExtend<T>: Default + Extend<T> {
    fn len(&self) -> usize;

    fn push_one(&mut self, value: T);
}
