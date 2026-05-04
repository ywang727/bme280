pub trait SensorState {}

#[derive(Debug)]
pub struct Sleep;
impl SensorState for Sleep {}

#[derive(Debug)]
pub struct Normal;
impl SensorState for Normal {}

#[derive(Debug)]
pub struct Forced;
impl SensorState for Forced {}
