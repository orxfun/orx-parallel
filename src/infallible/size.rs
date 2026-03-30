use crate::infallible::xap::Xap;

pub trait Size {}

pub struct One;

impl Size for One {}

pub struct Bin;

impl Size for Bin {}

pub struct Many;

impl Size for Many {}
