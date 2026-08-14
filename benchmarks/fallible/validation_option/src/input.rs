use orx_criterion::Factors;

#[derive(Debug, Clone, Copy)]
pub enum InvalidProfile {
    SuccessHeavy,
    Mixed,
    FailEarly,
}

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub profile: InvalidProfile,
}

impl InputVariant {
    pub fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "scenario"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.profile {
                InvalidProfile::SuccessHeavy => "success-heavy",
                InvalidProfile::Mixed => "mixed",
                InvalidProfile::FailEarly => "fail-early",
            }
            .to_string(),
        ]
    }
}
