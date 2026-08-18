use orx_criterion::Factors;

pub struct InputVariant {
    pub num_cities: usize,
    pub iterations: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["cities", "iterations"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("{}", self.num_cities),
            format!("{}", self.iterations),
        ]
    }
}
