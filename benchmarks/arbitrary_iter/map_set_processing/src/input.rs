use orx_criterion::Factors;

#[derive(Debug, Clone, Copy)]
pub enum Dataset {
    Map,
    Set,
}

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub dataset: Dataset,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "dataset"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.dataset {
                Dataset::Map => "hash-map",
                Dataset::Set => "hash-set",
            }
            .to_string(),
        ]
    }
}
