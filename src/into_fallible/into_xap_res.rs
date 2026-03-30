use crate::result::xap_res::XapRes;

pub trait IntoXapRes {
    type XapRes: XapRes;

    fn into_xap_res(self) -> Self::XapRes;
}
