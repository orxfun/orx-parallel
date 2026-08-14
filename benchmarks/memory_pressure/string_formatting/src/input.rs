use orx_criterion::Factors;

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub size: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["size"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self.size {
                10_000 => "small-10k",
                100_000 => "medium-100k",
                _ => "unknown",
            }
            .to_string(),
        ]
    }
}
