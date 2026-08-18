use orx_criterion::Factors;

pub struct InputVariant {
    pub num_items: usize,
    pub population_size: usize,
    pub heavy: bool,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["items", "population", "heavy"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("{}", self.num_items),
            format!("{}", self.population_size),
            format!("{}", self.heavy),
        ]
    }
}
