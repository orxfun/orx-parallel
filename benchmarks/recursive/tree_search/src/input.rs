use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub depth: usize,
    pub fan_out: usize,
    pub threshold: u64,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["depth", "fanout", "threshold"]
    }
    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.depth.to_string(),
            self.fan_out.to_string(),
            self.threshold.to_string(),
        ]
    }
}
