use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub heterogeneity_level: f64,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "het-lvl"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            format!("{:4}", self.heterogeneity_level),
        ]
    }
}
