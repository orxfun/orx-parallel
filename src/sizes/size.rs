pub trait Size: Clone + Copy + Send + Default {
    type ThenBin: Size;

    fn size() -> Option<usize>;
}
