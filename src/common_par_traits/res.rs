use crate::ParCollectInto;

pub trait ParResCommon {
    type CommonItem;

    type CommonError;

    fn common_collect_into<C>(self, dst: &mut C) -> Result<(), Self::CommonError>
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send,
        Self::CommonError: Send;
}
