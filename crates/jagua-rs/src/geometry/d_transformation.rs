use ordered_float::NotNan;

use crate::geometry::Transformation;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default)]
pub struct DTransformation {
    pub rotation: NotNan<f32>,
    pub translation: (NotNan<f32>, NotNan<f32>),
}

impl DTransformation {
    pub fn new(rotation: f32, translation: (f32, f32)) -> Self {
        Self {
            rotation: NotNan::new(rotation).expect("rotation is NaN"),
            translation: (
                NotNan::new(translation.0).expect("translation.0 is NaN"),
                NotNan::new(translation.1).expect("translation.1 is NaN"),
            ),
        }
    }

    pub const fn empty() -> Self {
        const Z: NotNan<f32> = unsafe { NotNan::new_unchecked(0.0) };
        Self {
            rotation: Z,
            translation: (Z, Z),
        }
    }

    pub fn rotation(&self) -> f32 {
        self.rotation.into()
    }

    pub fn translation(&self) -> (f32, f32) {
        (self.translation.0.into(), self.translation.1.into())
    }

    pub fn compose(&self) -> Transformation {
        Transformation {
            rotation: self.rotation(),
            translation: self.translation(),
        }
    }
}
