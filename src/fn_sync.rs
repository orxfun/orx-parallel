pub trait FnSync: Send + Sync + Clone {}

impl<X: Send + Sync + Clone> FnSync for X {}
