use crate::pool::BasicPool;

#[test]
fn basic_pool_drops_cleanly() {
    let _pool = BasicPool::new(1);
}
