#[derive(Clone)]
pub struct Node {
    pub id: usize,
    pub symbols: Vec<String>,
    pub children_symbols: Vec<String>,
}

impl Node {
    /// Fibonacci as example computation on each of the node values.
    pub fn compute(&self, amount_of_work: usize) -> u64 {
        let iter = Box::new(core::hint::black_box(0..amount_of_work));
        iter.map(|j| {
            let n = core::hint::black_box((self.id as u64 + j as u64) % 40);
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                let c = core::hint::black_box(a + b);
                a = b;
                b = c;
            }
            a
        })
        .sum()
    }
}
