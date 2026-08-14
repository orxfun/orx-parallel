use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub heavy: bool,
    pub heterogeneity_percent: u8,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "heavy", "het"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.n.to_string(),
            if self.heavy { "true" } else { "false" }.to_string(),
            format!("{}%", self.heterogeneity_percent),
        ]
    }
}
