use orx_criterion::Factors;

pub struct InputVariant {
    pub num_items: usize,
    pub restarts: usize,
    pub steps: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["items", "restarts", "steps"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.num_items.to_string(),
            self.restarts.to_string(),
            self.steps.to_string(),
        ]
    }
}
