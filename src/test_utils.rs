pub fn test_n_nt_chunk<T>(n: &[usize], nt: &[usize], chunk: &[usize], test: T)
where
    T: Fn(usize, usize, usize),
{
    for n in n {
        for nt in nt {
            for chunk in chunk {
                test(*n, *nt, *chunk);
            }
        }
    }
}
