use crate::geometry::Transformation;

pub trait Transformable: Clone {
    fn transform(&mut self, t: &Transformation) -> &mut Self;

    fn transform_clone(&self, t: &Transformation) -> Self {
        let mut c = self.clone();
        c.transform(t);
        c
    }
}
