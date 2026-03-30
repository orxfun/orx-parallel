pub trait Size {}

pub struct One;

impl Size for One {}

pub struct ZeroOne;

impl Size for ZeroOne {}

pub struct Many;

impl Size for Many {}
