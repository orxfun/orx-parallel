pub trait Count {}

pub struct ZeroOne;
impl Count for ZeroOne {}

pub struct One;
impl Count for One {}

pub struct Many;
impl Count for Many {}
