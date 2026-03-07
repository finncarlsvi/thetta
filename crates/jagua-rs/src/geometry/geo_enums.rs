#[derive(Clone, Debug, PartialEq)]
pub enum RotationRange {
    None,
    Continuous,
    Discrete(Vec<f32>),
}
