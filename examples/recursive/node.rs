#[derive(Clone)]
pub struct Node {
    pub id: usize,
    pub symbols: Vec<String>,
    pub children_symbols: Vec<String>,
}

impl Node {
    /// Fibonacci as example computation on each of the node values.
    pub fn compute(&self, amount_of_work: usize) -> u64 {
        (0..amount_of_work)
            .map(|j| {
                let n = core::hint::black_box(40 + self.id as u64 + j as u64);
                let mut a = 0;
                let mut b = 1;
                for _ in 0..n {
                    let c = a + b;
                    a = b;
                    b = c;
                }
                a
            })
            .sum()
    }
}
