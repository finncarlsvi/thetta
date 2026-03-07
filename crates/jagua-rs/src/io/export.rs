use crate::geometry::DTransformation;

pub fn int_to_ext_transformation(
    int_transf: &DTransformation,
    pre_transf: &DTransformation,
) -> DTransformation {
    let r = int_transf.rotation();
    let (tx, ty) = int_transf.translation();
    let (pre_tx, pre_ty) = pre_transf.translation();
    let centroid = (-pre_tx, -pre_ty);
    let (sin_r, cos_r) = r.sin_cos();
    let rotated_centroid = (
        centroid.0 * cos_r - centroid.1 * sin_r,
        centroid.0 * sin_r + centroid.1 * cos_r,
    );

    DTransformation::new(r, (tx - rotated_centroid.0, ty - rotated_centroid.1))
}
