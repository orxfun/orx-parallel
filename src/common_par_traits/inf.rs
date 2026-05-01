use crate::ParCollectInto;

pub trait ParInfCommon {
    type CommonItem;

    fn common_collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send;
}
