pub type Instant = std::time::Instant;

pub struct Timing;

impl Timing {
    #[inline(always)]
    pub fn now() -> Instant {
        Instant::now()
    }

    #[inline(always)]
    pub fn elapsed_ns_from(from: Instant) -> u64 {
        from.elapsed().as_nanos().min(u64::MAX as u128) as u64
    }

    #[inline(always)]
    pub fn elapsed_millis_from(from: Instant) -> u128 {
        from.elapsed().as_millis()
    }
}
