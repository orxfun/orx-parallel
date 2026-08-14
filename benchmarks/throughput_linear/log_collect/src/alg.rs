use orx_criterion::Factors;

#[allow(dead_code)]
pub enum Method { Seq, Rayon, OrxOnce, OrxBasic, OrxRayon }

impl Method {
    #[allow(unreachable_code)]
    pub fn get() -> Self {
        #[cfg(feature = "seq")] return Self::Seq;
        #[cfg(feature = "rayon")] return Self::Rayon;
        #[cfg(feature = "orx-once")] return Self::OrxOnce;
        #[cfg(feature = "orx-basic")] return Self::OrxBasic;
        #[cfg(feature = "orx-rayon")] return Self::OrxRayon;
        panic!("must add one of the algorithm variants as feature");
    }
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> { vec!["method"] }
    fn factor_levels(&self) -> Vec<String> {
        vec![match self { Self::Seq => "seq", Self::Rayon => "rayon", Self::OrxOnce => "orx-once", Self::OrxBasic => "orx-basic", Self::OrxRayon => "orx-rayon" }.to_string()]
    }
}
