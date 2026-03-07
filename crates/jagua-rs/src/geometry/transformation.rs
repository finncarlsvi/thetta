#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transformation {
    pub rotation: f32,
    pub translation: (f32, f32),
}

impl Transformation {
    pub const fn empty() -> Self {
        Self {
            rotation: 0.0,
            translation: (0.0, 0.0),
        }
    }

    pub fn inverse(&self) -> Self {
        let r = -self.rotation;
        let (sin_r, cos_r) = r.sin_cos();
        let (tx, ty) = self.translation;
        let itx = -(tx * cos_r - ty * sin_r);
        let ity = -(tx * sin_r + ty * cos_r);
        Self {
            rotation: r,
            translation: (itx, ity),
        }
    }

    pub fn transform(&self, other: &Transformation) -> Self {
        // Apply self, then other.
        let rotation = self.rotation + other.rotation;
        let (sin_b, cos_b) = other.rotation.sin_cos();
        let (ax, ay) = self.translation;
        let rotated_a = (ax * cos_b - ay * sin_b, ax * sin_b + ay * cos_b);
        Self {
            rotation,
            translation: (rotated_a.0 + other.translation.0, rotated_a.1 + other.translation.1),
        }
    }

    pub fn transform_from_decomposed(
        &self,
        d: &crate::geometry::d_transformation::DTransformation,
    ) -> Self {
        self.transform(&d.compose())
    }

    pub fn decompose(&self) -> crate::geometry::d_transformation::DTransformation {
        crate::geometry::d_transformation::DTransformation::new(self.rotation, self.translation)
    }
}
