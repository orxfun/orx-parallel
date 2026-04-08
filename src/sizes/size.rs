pub trait Size: Clone + Copy + Send + Default {
    type ThenBin: Size;
}
