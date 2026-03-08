use anyhow::{Result, bail};

use crate::collision_detection::CDEConfig;
use crate::entities::Item;
use crate::geometry::DTransformation;
use crate::geometry::OriginalShape;
use crate::geometry::geo_enums::RotationRange;
use crate::geometry::primitives::{Point, Rect, SPolygon};
use crate::geometry::shape_modification::ShapeModifyConfig;
use crate::io::ext_repr::{ExtItem, ExtSPolygon, ExtShape};

#[derive(Clone, Debug, Copy)]
pub struct Importer {
    pub shape_modify_config: ShapeModifyConfig,
    pub cde_config: CDEConfig,
}

impl Importer {
    pub fn new(
        cde_config: CDEConfig,
        simplify_tolerance: Option<f32>,
        min_item_separation: Option<f32>,
        narrow_concavity_cutoff: Option<(f32, f32)>,
    ) -> Self {
        Self {
            shape_modify_config: ShapeModifyConfig {
                offset: min_item_separation.map(|f| f / 2.0),
                simplify_tolerance,
                narrow_concavity_cutoff,
            },
            cde_config,
        }
    }

    pub fn import_item(&self, ext_item: &ExtItem) -> Result<Item> {
        let shape = match &ext_item.shape {
            ExtShape::Rectangle {
                x_min,
                y_min,
                width,
                height,
            } => SPolygon::new(vec![
                Point(*x_min, *y_min),
                Point(*x_min + *width, *y_min),
                Point(*x_min + *width, *y_min + *height),
                Point(*x_min, *y_min + *height),
            ])?,
            ExtShape::SimplePolygon(sp) => import_simple_polygon(sp)?,
            ExtShape::Polygon(p) => import_simple_polygon(&p.outer)?,
            ExtShape::MultiPolygon(_) => bail!("No support for multipolygons yet"),
        };

        let original_shape = OriginalShape {
            pre_transform: centering_transformation(&shape),
            shape,
            modify_config: self.shape_modify_config,
        };

        let allowed_orientations = match ext_item.allowed_orientations.as_ref() {
            Some(a_o) => {
                if a_o.is_empty() || (a_o.len() == 1 && a_o[0] == 0.0) {
                    RotationRange::None
                } else {
                    RotationRange::Discrete(a_o.iter().map(|angle| angle.to_radians()).collect())
                }
            }
            None => RotationRange::Continuous,
        };

        Item::new(
            ext_item.id as usize,
            original_shape,
            allowed_orientations,
            ext_item.min_quality,
            self.cde_config.item_surrogate_config,
        )
    }
}

pub fn import_simple_polygon(sp: &ExtSPolygon) -> Result<SPolygon> {
    let mut points = sp.0.iter().map(|(x, y)| Point(*x, *y)).collect::<Vec<_>>();
    if points.len() > 1 && points[0] == points[points.len() - 1] {
        points.pop();
    }
    if points.len() < 3 {
        bail!("simple polygon requires at least 3 vertices")
    }
    SPolygon::new(points)
}

pub fn centering_transformation(shape: &SPolygon) -> DTransformation {
    let Point(cx, cy) = shape.centroid();
    DTransformation::new(0.0, (-cx, -cy))
}

pub fn ext_to_int_transformation(
    ext_transf: &DTransformation,
    pre_transf: &DTransformation,
) -> DTransformation {
    let r = ext_transf.rotation();
    let (tx, ty) = ext_transf.translation();
    let (pre_tx, pre_ty) = pre_transf.translation();
    let centroid = (-pre_tx, -pre_ty);
    let (sin_r, cos_r) = r.sin_cos();
    let rotated_centroid = (
        centroid.0 * cos_r - centroid.1 * sin_r,
        centroid.0 * sin_r + centroid.1 * cos_r,
    );

    DTransformation::new(r, (tx + rotated_centroid.0, ty + rotated_centroid.1))
}

#[allow(dead_code)]
fn _dummy_rect_use(_r: Rect) {}
