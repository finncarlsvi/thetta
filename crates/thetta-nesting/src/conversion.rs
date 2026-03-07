use anyhow::{Result, bail, ensure};
use jagua_rs::collision_detection::CDEConfig;
use jagua_rs::entities::Instance;
use jagua_rs::geometry::DTransformation;
use jagua_rs::geometry::fail_fast::SPSurrogateConfig;
use jagua_rs::geometry::geo_enums::RotationRange;
use jagua_rs::io::export::int_to_ext_transformation;
use jagua_rs::io::ext_repr::{ExtItem, ExtSPolygon, ExtShape};
use jagua_rs::io::import::{Importer, ext_to_int_transformation};
use jagua_rs::probs::spp::entities::SPInstance;
use jagua_rs::probs::spp::io::ext_repr::{ExtItem as ExtSPItem, ExtSPInstance};
use jagua_rs::probs::spp::io::import_instance;
use std::f64::consts::TAU;
use thetta_geometry::types::{Piece, Polygon, SheetConfig};

const DEFAULT_CDE_CONFIG: CDEConfig = CDEConfig {
    quadtree_depth: 4,
    cd_threshold: 16,
    item_surrogate_config: SPSurrogateConfig {
        n_pole_limits: [(64, 0.0), (16, 0.8), (8, 0.9)],
        n_ff_poles: 1,
        n_ff_piers: 0,
    },
};

pub fn pieces_to_sp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<SPInstance> {
    ensure!(!pieces.is_empty(), "at least one piece is required");
    ensure!(sheet.height.is_finite() && sheet.height > 0.0, "sheet.height must be > 0");

    if let Some(per_piece) = orientations {
        ensure!(
            per_piece.len() == pieces.len(),
            "orientations length ({}) must match pieces length ({})",
            per_piece.len(),
            pieces.len()
        );
    }

    let ext_items = pieces
        .iter()
        .enumerate()
        .map(|(idx, piece)| {
            let shape = polygon_to_ext_simple_polygon(&piece.polygon)?;
            let allowed_orientations = orientations
                .and_then(|ori| ori.get(idx))
                .map(|angles| angles.iter().map(|r| r.to_degrees() as f32).collect());

            Ok(ExtSPItem {
                base: ExtItem {
                    id: idx as u64,
                    allowed_orientations,
                    shape: ExtShape::SimplePolygon(shape),
                    min_quality: None,
                },
                demand: 1,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let ext_instance = ExtSPInstance {
        name: "thetta".to_string(),
        items: ext_items,
        strip_height: sheet.height as f32,
    };

    let importer = Importer::new(
        DEFAULT_CDE_CONFIG,
        None,
        (sheet.separation > 0.0).then_some(sheet.separation as f32),
        None,
    );

    import_instance(&importer, &ext_instance)
}

pub fn default_orientations(len: usize) -> Vec<RotationRange> {
    (0..len).map(|_| RotationRange::Continuous).collect()
}

pub fn rotation_range_to_samples(rotation_range: &RotationRange, rotations_n: usize) -> Vec<f64> {
    match rotation_range {
        RotationRange::None => vec![0.0],
        RotationRange::Continuous => {
            let n = rotations_n.max(1);
            (0..n).map(|i| i as f64 * TAU / n as f64).collect()
        }
        RotationRange::Discrete(angles) => {
            if angles.is_empty() {
                vec![0.0]
            } else {
                angles.iter().map(|a| *a as f64).collect()
            }
        }
    }
}

pub fn to_internal_transformation(
    instance: &SPInstance,
    item_id: usize,
    x: f64,
    y: f64,
    rotation: f64,
) -> DTransformation {
    let ext = DTransformation::new(rotation as f32, (x as f32, y as f32));
    let pre = &instance.item(item_id).shape_orig.pre_transform;
    ext_to_int_transformation(&ext, pre)
}

pub fn to_external_transformation(
    instance: &SPInstance,
    item_id: usize,
    int_transf: &DTransformation,
) -> DTransformation {
    let pre = &instance.item(item_id).shape_orig.pre_transform;
    int_to_ext_transformation(int_transf, pre)
}

fn polygon_to_ext_simple_polygon(poly: &Polygon) -> Result<ExtSPolygon> {
    ensure!(poly.holes.is_empty(), "holes are not supported yet");
    ensure!(poly.outer.len() >= 3, "polygon must have at least 3 vertices");

    let mut points = poly
        .outer
        .iter()
        .map(|p| (p.x as f32, p.y as f32))
        .collect::<Vec<_>>();

    if points.len() >= 2 && points.first() == points.last() {
        points.pop();
    }

    if points.len() < 3 {
        bail!("polygon must have at least 3 distinct vertices after cleanup");
    }

    Ok(ExtSPolygon(points))
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::polygon_ops;
    use thetta_geometry::test_shapes;
    use thetta_geometry::types::{PieceId, Point2D};

    fn rect_piece(id: u64, w: f64, h: f64) -> Piece {
        Piece {
            id: PieceId(id),
            polygon: test_shapes::rectangle(w, h),
            source_file: "test.json".into(),
            quantity: 1,
        }
    }

    #[test]
    fn f64_to_f32_preserves_area_within_tolerance() {
        let pieces = vec![rect_piece(100, 10.123_456_7, 20.765_432_1)];
        let sheet = SheetConfig {
            width: 1000.0,
            height: 1000.0,
            separation: 0.0,
        };
        let instance = pieces_to_sp_instance(&pieces, &sheet, None).unwrap();
        let item = instance.item(0);

        let expected = polygon_ops::area(&pieces[0].polygon) as f32;
        let actual = item.shape_orig.area();
        assert!((expected - actual).abs() < 1e-3);
    }

    #[test]
    fn separation_inflates_bbox() {
        let pieces = vec![rect_piece(7, 10.0, 20.0)];
        let sheet_no_sep = SheetConfig {
            width: 100.0,
            height: 100.0,
            separation: 0.0,
        };
        let sheet_sep = SheetConfig {
            width: 100.0,
            height: 100.0,
            separation: 4.0,
        };

        let no_sep = pieces_to_sp_instance(&pieces, &sheet_no_sep, None).unwrap();
        let sep = pieces_to_sp_instance(&pieces, &sheet_sep, None).unwrap();

        let w0 = no_sep.item(0).shape_cd.bbox.width();
        let w1 = sep.item(0).shape_cd.bbox.width();
        assert!(w1 > w0);
    }

    #[test]
    fn orientation_mapping_variants() {
        let pieces = vec![rect_piece(1, 5.0, 5.0), rect_piece(2, 5.0, 5.0), rect_piece(3, 5.0, 5.0)];
        let sheet = SheetConfig {
            width: 100.0,
            height: 100.0,
            separation: 0.0,
        };
        let orientations = vec![vec![0.0], vec![], vec![0.0, std::f64::consts::FRAC_PI_2]];
        let instance = pieces_to_sp_instance(&pieces, &sheet, Some(&orientations)).unwrap();

        assert!(matches!(instance.item(0).allowed_rotation, RotationRange::None));
        assert!(matches!(instance.item(1).allowed_rotation, RotationRange::None));
        assert!(matches!(instance.item(2).allowed_rotation, RotationRange::Discrete(_)));
    }

    #[test]
    fn renumbers_non_consecutive_piece_ids() {
        let pieces = vec![rect_piece(10, 3.0, 3.0), rect_piece(42, 4.0, 4.0)];
        let sheet = SheetConfig {
            width: 100.0,
            height: 100.0,
            separation: 0.0,
        };
        let instance = pieces_to_sp_instance(&pieces, &sheet, None).unwrap();

        assert_eq!(instance.item(0).id, 0);
        assert_eq!(instance.item(1).id, 1);
    }

    #[test]
    fn strips_closed_ring_vertex() {
        let mut poly = test_shapes::triangle(10.0, 10.0);
        poly.outer.push(Point2D { x: 0.0, y: 0.0 });
        let pieces = vec![Piece {
            id: PieceId(1),
            polygon: poly,
            source_file: "test.json".into(),
            quantity: 1,
        }];
        let sheet = SheetConfig {
            width: 100.0,
            height: 100.0,
            separation: 0.0,
        };

        let instance = pieces_to_sp_instance(&pieces, &sheet, None).unwrap();
        assert_eq!(instance.item(0).shape_orig.shape.n_vertices(), 3);
    }
}
