pub fn sort_if_ok<T: Ord + PartialOrd, E>(res_vec: Result<Vec<T>, E>) -> Result<Vec<T>, E> {
    res_vec.map(|mut x| {
        x.sort();
        x
    })
}
