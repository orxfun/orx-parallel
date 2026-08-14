use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub with_trace: bool,
}

impl InputVariant {
    pub fn len(&self) -> usize {
        1 << self.n
    }

    pub fn steps(&self) -> usize {
        if self.with_trace { 192 } else { 96 }
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "mode"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            if self.with_trace { "stats+trace" } else { "stats" }.to_string(),
        ]
    }
}
