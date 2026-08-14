use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n)]
    }
}
