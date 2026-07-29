#[derive(Clone, Copy)]
pub struct EwmaParams {
    pub numerator: u64,
    pub denominator: u64,
}

impl EwmaParams {
    #[inline]
    pub fn ewma(self, previous: u64, sample: u64) -> u64 {
        match previous {
            0 => sample,
            value => (value.saturating_mul(self.numerator) + sample) / self.denominator,
        }
    }
}
