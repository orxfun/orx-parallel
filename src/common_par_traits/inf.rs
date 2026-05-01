use crate::ParCollectInto;

pub trait ParInfCommon {
    type InfItem;

    fn inf_collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::InfItem>,
        Self::InfItem: Send;
}
