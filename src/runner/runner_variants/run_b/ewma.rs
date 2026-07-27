#[derive(Clone, Copy)]
pub struct EwmaParams {
    numerator: u64,
    denominator: u64,
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

pub const EWMA_PARAMS_AVG: EwmaParams = EwmaParams {
    numerator: 7,
    denominator: 8,
};

pub const EWMA_PARAMS_DEV: EwmaParams = EwmaParams {
    numerator: 3,
    denominator: 4,
};
