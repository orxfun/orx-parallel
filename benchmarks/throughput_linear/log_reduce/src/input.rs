use orx_criterion::Factors;

pub struct InputVariant { pub n: usize, pub heavy: bool }

impl InputVariant { pub fn len(&self) -> usize { 1 << self.n } }

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> { vec!["n", "task"] }
    fn factor_levels(&self) -> Vec<String> { vec![format!("2e{}", self.n), if self.heavy { "heavy" } else { "light" }.to_string()] }
}
