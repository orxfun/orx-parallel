use crate::ParCollectInto;

pub trait ParOptCommon {
    type CommonItem;

    fn common_collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send;
}
