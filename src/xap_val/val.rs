use crate::xap_val::Flow;

pub trait Val {
    type Flow: Flow;
}
