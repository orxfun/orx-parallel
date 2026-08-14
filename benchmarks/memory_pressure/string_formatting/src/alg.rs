use orx_criterion::Factors;

#[allow(dead_code)]
pub enum Method {
    Seq,
    Rayon,
    OrxOnce,
    OrxBasic,
    OrxRayon,
}

impl Method {
    #[allow(unreachable_code)]
    pub fn get() -> Self {
        #[cfg(feature = "seq")]
        return Self::Seq;

        #[cfg(feature = "rayon")]
        return Self::Rayon;

        #[cfg(feature = "orx-once")]
        return Self::OrxOnce;

        #[cfg(feature = "orx-basic")]
        return Self::OrxBasic;

        #[cfg(feature = "orx-rayon")]
        return Self::OrxRayon;

        panic!("must add one of the variants algorithm variants as feature");
    }
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon => format!("rayon"),
            Self::OrxOnce => format!("orx-once"),
            Self::OrxBasic => format!("orx-basic"),
            Self::OrxRayon => format!("orx-rayon"),
        }]
    }
}
