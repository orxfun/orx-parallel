pub type Instant = u64;

pub struct Timing;

impl Timing {
    #[inline(always)]
    pub fn now() -> Instant {
        (js_sys::Date::now() * 1_000_000.0) as u64
    }

    #[inline(always)]
    pub fn elapsed_ns_from(from: Instant) -> u64 {
        Timing::now().saturating_sub(from)
    }

    #[inline(always)]
    pub fn elapsed_millis_from(from: Instant) -> u128 {
        Timing::now().saturating_sub(from).saturating_div(1_000_000) as u128
    }
}
