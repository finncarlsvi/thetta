use anyhow::{Context, Result, bail, ensure};
use jagua_rs::geometry::geo_enums::RotationRange;
use jagua_rs::io::ext_repr::{ExtSPolygon, ExtShape};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::f64::consts::TAU;
use std::path::Path;
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{Piece, PieceId, Point2D, Polygon, SheetConfig};

#[derive(Debug, Deserialize)]
pub struct BenchmarkInstance {
    pub name: String,
    pub items: Vec<BenchmarkItem>,
    pub strip_height: f64,
}

#[derive(Debug, Deserialize)]
pub struct BenchmarkItem {
    pub id: u64,
    pub demand: u64,
    pub allowed_orientations: Option<Vec<f64>>,
    pub shape: BenchmarkShape,
}

pub type BenchmarkShape = ExtShape;

#[derive(Debug, Clone)]
pub struct BenchmarkMeta {
    pub name: String,
    pub strip_height: f64,
    pub orientations: Vec<RotationRange>,
    pub known_best_strip_length: Option<f64>,
}

pub fn load_benchmark(path: &Path) -> Result<(Vec<Piece>, SheetConfig, BenchmarkMeta)> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: BenchmarkInstance = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse benchmark json {}", path.display()))?;

    ensure!(
        parsed.strip_height.is_finite() && parsed.strip_height > 0.0,
        "strip_height must be > 0"
    );

    let mut sorted_items = parsed.items;
    sorted_items.sort_by_key(|i| i.id);

    let mut pieces = Vec::new();
    let mut orientations = Vec::new();

    for item in sorted_items {
        let polygon = shape_to_polygon(&item.shape)
            .with_context(|| format!("invalid shape for benchmark item {}", item.id))?;
        let orientation = normalize_rotation_range(item.allowed_orientations.as_deref());

        for _ in 0..item.demand {
            let next_id = pieces.len() as u64;
            pieces.push(Piece {
                id: PieceId(next_id),
                polygon: polygon.clone(),
                source_file: path.to_path_buf(),
                quantity: 1,
            });
            orientations.push(orientation.clone());
        }
    }

    ensure!(!pieces.is_empty(), "benchmark must contain at least one demanded piece");

    let total_area = pieces
        .iter()
        .map(|piece| polygon_ops::area(&piece.polygon))
        .sum::<f64>();
    let width_lb = (total_area / parsed.strip_height).max(1.0);

    let sheet = SheetConfig {
        width: width_lb,
        height: parsed.strip_height,
        separation: 0.0,
    };

    let meta = BenchmarkMeta {
        name: parsed.name.clone(),
        strip_height: parsed.strip_height,
        orientations,
        known_best_strip_length: known_best(&parsed.name),
    };

    Ok((pieces, sheet, meta))
}

pub fn known_best(instance: &str) -> Option<f64> {
    let table: BTreeMap<&str, f64> = [
        ("albano", 9692.056),
        ("blaz1", 25.049),
        ("dagli", 56.087),
        ("fu", 30.843),
        ("gardeyn0", 48681.078),
        ("gardeyn1", 16181.270),
        ("gardeyn2", 47382.848),
        ("gardeyn3", 60397.832),
        ("gardeyn4", 4733.340),
        ("gardeyn5", 4322.151),
        ("gardeyn6", 17061.811),
        ("gardeyn7", 5067.831),
        ("gardeyn8", 48718.980),
        ("gardeyn9", 65.793),
        ("jakobs1", 10.980),
        ("jakobs2", 22.000),
        ("mao", 1696.802),
        ("marques", 75.176),
        ("shapes0", 57.012),
        ("shapes1", 52.002),
        ("shirts", 59.393),
        ("swim", 5541.653),
        ("trousers", 235.172),
    ]
    .into_iter()
    .collect();

    table.get(instance).copied()
}

fn shape_to_polygon(shape: &BenchmarkShape) -> Result<Polygon> {
    match shape {
        ExtShape::Rectangle {
            x_min,
            y_min,
            width,
            height,
        } => {
            ensure!(*width > 0.0 && *height > 0.0, "rectangle must have positive dimensions");
            Ok(Polygon {
                outer: vec![
                    Point2D {
                        x: *x_min as f64,
                        y: *y_min as f64,
                    },
                    Point2D {
                        x: (*x_min + *width) as f64,
                        y: *y_min as f64,
                    },
                    Point2D {
                        x: (*x_min + *width) as f64,
                        y: (*y_min + *height) as f64,
                    },
                    Point2D {
                        x: *x_min as f64,
                        y: (*y_min + *height) as f64,
                    },
                ],
                holes: vec![],
            })
        }
        ExtShape::SimplePolygon(sp) => Ok(Polygon {
            outer: ring_from_ext(sp)?,
            holes: vec![],
        }),
        ExtShape::Polygon(poly) => Ok(Polygon {
            outer: ring_from_ext(&poly.outer)?,
            holes: poly
                .inner
                .iter()
                .map(ring_from_ext)
                .collect::<Result<Vec<_>>>()?,
        }),
        ExtShape::MultiPolygon(_) => bail!("multipolygon benchmark items are not supported"),
    }
}

fn ring_from_ext(sp: &ExtSPolygon) -> Result<Vec<Point2D>> {
    let mut pts = sp
        .0
        .iter()
        .map(|(x, y)| Point2D {
            x: *x as f64,
            y: *y as f64,
        })
        .collect::<Vec<_>>();

    if pts.len() >= 2 && pts.first() == pts.last() {
        pts.pop();
    }

    ensure!(pts.len() >= 3, "polygon ring must have at least 3 vertices");
    Ok(pts)
}

fn normalize_rotation_range(angles_deg: Option<&[f64]>) -> RotationRange {
    match angles_deg {
        None => RotationRange::Continuous,
        Some([]) => RotationRange::None,
        Some(values) => {
            let mut normalized = values
                .iter()
                .map(|deg| normalize_angle(deg.to_radians()) as f32)
                .collect::<Vec<_>>();

            normalized.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            normalized.dedup_by(|a, b| (*a as f64 - *b as f64).abs() < 1e-6);

            if normalized.len() <= 1 {
                RotationRange::None
            } else {
                RotationRange::Discrete(normalized)
            }
        }
    }
}

fn normalize_angle(mut angle: f64) -> f64 {
    angle %= TAU;
    if angle < 0.0 {
        angle += TAU;
    }
    angle
}
