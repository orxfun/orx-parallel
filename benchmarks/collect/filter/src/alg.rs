use orx_criterion::Factors;

#[allow(dead_code)]
pub enum Method {
    Seq,
    Rayon,
    RayonVec2,
    OrxOnce,
    OrxBasic,
    OrxRayon,
    OrxOnceVec2,
    OrxBasicVec2,
    OrxRayonVec2,
}

impl Method {
    #[allow(unreachable_code)]
    pub fn get() -> Self {
        #[cfg(feature = "seq")]
        return Self::Seq;

        #[cfg(feature = "rayon")]
        return Self::Rayon;

        #[cfg(feature = "rayon-vec2")]
        return Self::RayonVec2;

        #[cfg(feature = "orx-once")]
        return Self::OrxOnce;

        #[cfg(feature = "orx-basic")]
        return Self::OrxBasic;

        #[cfg(feature = "orx-rayon")]
        return Self::OrxRayon;

        #[cfg(feature = "orx-once-vec2")]
        return Self::OrxOnceVec2;

        #[cfg(feature = "orx-basic-vec2")]
        return Self::OrxBasicVec2;

        #[cfg(feature = "orx-rayon-vec2")]
        return Self::OrxRayonVec2;

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
            Self::RayonVec2 => format!("rayon-vec2"),
            Self::OrxOnce => format!("orx-once"),
            Self::OrxBasic => format!("orx-basic"),
            Self::OrxRayon => format!("orx-rayon"),
            Self::OrxOnceVec2 => format!("orx-once-vec2"),
            Self::OrxBasicVec2 => format!("orx-basic-vec2"),
            Self::OrxRayonVec2 => format!("orx-rayon-vec2"),
        }]
    }
}
