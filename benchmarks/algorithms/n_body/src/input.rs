use orx_criterion::Factors;

pub struct InputVariant {
    pub num_bodies: usize,
    pub steps: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["bodies", "steps"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.num_bodies.to_string(), self.steps.to_string()]
    }
}
