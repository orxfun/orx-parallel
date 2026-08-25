use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub depth: usize,
    pub fanout: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["depth", "fanout"]
    }
    fn factor_levels(&self) -> Vec<String> {
        vec![self.depth.to_string(), self.fanout.to_string()]
    }
}
