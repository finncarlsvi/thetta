#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ShapeModifyConfig {
    pub offset: Option<f32>,
    pub simplify_tolerance: Option<f32>,
    pub narrow_concavity_cutoff: Option<(f32, f32)>,
}
