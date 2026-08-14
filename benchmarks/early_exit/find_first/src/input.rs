use orx_criterion::Factors;

#[derive(Debug, Clone, Copy)]
pub enum Pos {
    Early,
    Mid,
    Late,
    Never,
}

#[derive(Clone, Copy)]
pub struct InputVariant {
    pub n: usize,
    pub pos: Pos,
}

impl InputVariant {
    pub fn len(&self) -> usize {
        1 << self.n
    }
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "pos"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n),
            match self.pos {
                Pos::Early => "early",
                Pos::Mid => "mid",
                Pos::Late => "late",
                Pos::Never => "never",
            }
            .to_string(),
        ]
    }
}
