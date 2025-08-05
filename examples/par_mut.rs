fn main() {
    let mut vec = vec![1, 2, 3];
    let slice = vec.as_mut_slice();

    slice.iter_mut().for_each(|x| *x += 10);
    dbg!(vec);
}
