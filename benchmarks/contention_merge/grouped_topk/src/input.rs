use orx_criterion::Factors;

#[derive(Debug, Clone, Copy)]
pub enum Dist {
    Uniform,
    Skewed,
}

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub dist: Dist,
}

impl InputVariant {
    pub fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dist"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dist {
                Dist::Uniform => "uniform",
                Dist::Skewed => "skewed",
            }
            .to_string(),
        ]
    }
}
