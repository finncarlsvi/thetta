use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thetta_geometry::types::{Piece, PieceId, Point2D, Polygon};

use crate::profile::normalize_polygon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepSolidKey(pub u32);

#[derive(Debug, Clone)]
pub struct StepPieceRef {
    pub piece_id: PieceId,
    pub solid_key: StepSolidKey,
    pub source_path: PathBuf,
    pub thickness_mm: f64,
}

#[derive(Debug, Clone)]
pub struct ThicknessInfo {
    pub value_mm: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ExtractionConfig {
    pub mesh_tolerance_mm: f64,
    pub thickness_bucket_mm: f64,
    pub max_points_per_edge: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            mesh_tolerance_mm: 0.5,
            thickness_bucket_mm: 0.25,
            max_points_per_edge: 200,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepImportResult {
    pub model: StepModel,
    pub pieces: Vec<Piece>,
    pub piece_refs: Vec<StepPieceRef>,
    pub thicknesses: Vec<ThicknessInfo>,
    pub selected_thickness_mm: f64,
}

#[derive(Debug, Clone)]
pub struct StepModel {
    source_path: PathBuf,
    solids: Vec<SolidRecord>,
}

#[derive(Debug, Clone)]
struct SolidRecord {
    key: StepSolidKey,
    thickness_mm: f64,
    profile: Polygon,
    /// 3D coordinate frame from profile extraction + 2D centering offset.
    /// `None` for JSON-imported solids (which don't have 3D geometry).
    frame: Option<SolidFrame>,
}

/// Stores the profile extraction frame and 2D centering offset so that
/// the STEP exporter can reconstruct the correct 3D transformation.
#[derive(Debug, Clone)]
pub struct SolidFrame {
    /// The 3D coordinate frame the profile was extracted from.
    #[cfg(feature = "occt")]
    pub frame_3d: crate::occt::ProfileFrame3D,
    /// Bounding-box center of the 2D profile before centering.
    pub center_2d: (f64, f64),
}

impl StepModel {
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn solid_count(&self) -> usize {
        self.solids.len()
    }

    pub fn solid_profile(&self, key: StepSolidKey) -> Option<&Polygon> {
        self.solids.get(key.0 as usize).map(|s| &s.profile)
    }

    pub fn solid_frame(&self, key: StepSolidKey) -> Option<&SolidFrame> {
        self.solids
            .get(key.0 as usize)
            .and_then(|s| s.frame.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum StepImportError {
    FileNotFound,
    ReadFailed(String),
    NoSolidsFound,
    NoProfilesExtracted { skipped: usize, reason: String },
    OcctError(String),
}

impl Display for StepImportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound => write!(f, "file not found"),
            Self::ReadFailed(msg) => write!(f, "read failed: {msg}"),
            Self::NoSolidsFound => write!(f, "no solids found"),
            Self::NoProfilesExtracted { skipped, reason } => {
                write!(f, "no profiles extracted (skipped {skipped}): {reason}")
            }
            Self::OcctError(msg) => write!(f, "occt error: {msg}"),
        }
    }
}

impl std::error::Error for StepImportError {}

pub fn import_step(
    path: &Path,
    thickness: Option<f64>,
    config: &ExtractionConfig,
) -> Result<StepImportResult, StepImportError> {
    if !path.exists() {
        return Err(StepImportError::FileNotFound);
    }

    #[cfg(feature = "occt")]
    let is_step_file = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("step") || ext.eq_ignore_ascii_case("stp"));

    #[cfg(feature = "occt")]
    if is_step_file {
        return import_step_occt(path, thickness, config);
    }

    // Fallback: JSON-based import (for tests and non-OCCT builds)
    import_step_json(path, thickness, config)
}

fn import_step_json(
    path: &Path,
    thickness: Option<f64>,
    config: &ExtractionConfig,
) -> Result<StepImportResult, StepImportError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| StepImportError::ReadFailed(e.to_string()))?;
    let model = parse_step_model(path, &text)?;
    if model.solids.is_empty() {
        return Err(StepImportError::NoSolidsFound);
    }

    let thicknesses = detect_thickness_groups(&model, config.thickness_bucket_mm);
    let selected_thickness_mm =
        select_thickness(thickness, &thicknesses).ok_or(StepImportError::NoSolidsFound)?;

    let (pieces, piece_refs) = extract_pieces_for_thickness(&model, selected_thickness_mm, config)?;

    Ok(StepImportResult {
        model,
        pieces,
        piece_refs,
        thicknesses,
        selected_thickness_mm,
    })
}

#[cfg(feature = "occt")]
fn import_step_occt(
    path: &Path,
    thickness: Option<f64>,
    config: &ExtractionConfig,
) -> Result<StepImportResult, StepImportError> {
    use crate::occt;

    let reader = occt::OcctStepReader::read_file(path)?;
    let solid_infos = occt::enumerate_solids(reader.root_shape());

    if solid_infos.is_empty() {
        return Err(StepImportError::NoSolidsFound);
    }

    // Build SolidRecords by extracting 2D profiles from each solid
    let angular_deflection = 0.1; // radians
    let curvature_deflection = config.mesh_tolerance_mm;
    let mut solids = Vec::with_capacity(solid_infos.len());
    let mut skipped = 0usize;

    for (idx, info) in solid_infos.iter().enumerate() {
        let profile_opt =
            occt::extract_profile(&info.solid, angular_deflection, curvature_deflection);

        let (profile, frame) = match profile_opt {
            Some(fp) => {
                let frame_3d = fp.frame.clone();
                let polygon = Polygon {
                    outer: fp.outer,
                    holes: fp.holes,
                };

                // Compute 2D bbox center before normalize_polygon centers it
                let center_2d = bbox_center_2d(&polygon.outer);

                match normalize_polygon(polygon) {
                    Ok(p) if p.outer.len() >= 3 => (
                        p,
                        Some(SolidFrame {
                            frame_3d,
                            center_2d,
                        }),
                    ),
                    _ => {
                        skipped += 1;
                        continue;
                    }
                }
            }
            None => {
                skipped += 1;
                continue;
            }
        };

        solids.push(SolidRecord {
            key: StepSolidKey(idx as u32),
            thickness_mm: info.thickness_mm,
            profile,
            frame,
        });
    }

    if solids.is_empty() {
        return Err(StepImportError::NoProfilesExtracted {
            skipped,
            reason: "no planar faces found in any solid".into(),
        });
    }

    let model = StepModel {
        source_path: path.to_path_buf(),
        solids,
    };

    let thicknesses = detect_thickness_groups(&model, config.thickness_bucket_mm);
    let selected_thickness_mm =
        select_thickness(thickness, &thicknesses).ok_or(StepImportError::NoSolidsFound)?;

    let (pieces, piece_refs) = extract_pieces_for_thickness(&model, selected_thickness_mm, config)?;

    Ok(StepImportResult {
        model,
        pieces,
        piece_refs,
        thicknesses,
        selected_thickness_mm,
    })
}

pub fn extract_pieces_for_thickness(
    model: &StepModel,
    selected_thickness_mm: f64,
    config: &ExtractionConfig,
) -> Result<(Vec<Piece>, Vec<StepPieceRef>), StepImportError> {
    let mut pieces = Vec::new();
    let mut refs = Vec::new();
    let bucket = thickness_bucket(selected_thickness_mm, config.thickness_bucket_mm);
    let mut skipped = 0usize;

    for (solid_index, solid) in model.solids.iter().enumerate() {
        if thickness_bucket(solid.thickness_mm, config.thickness_bucket_mm) != bucket {
            continue;
        }

        if solid.profile.outer.len() < 3 {
            skipped += 1;
            continue;
        }

        let piece_id = PieceId(solid_index as u64);
        pieces.push(Piece {
            id: piece_id,
            polygon: solid.profile.clone(),
            source_file: model.source_path.clone(),
            quantity: 1,
        });
        refs.push(StepPieceRef {
            piece_id,
            solid_key: solid.key,
            source_path: model.source_path.clone(),
            thickness_mm: solid.thickness_mm,
        });
    }

    if pieces.is_empty() {
        return Err(StepImportError::NoProfilesExtracted {
            skipped,
            reason: "no solids matched selected thickness".to_string(),
        });
    }

    Ok((pieces, refs))
}

fn detect_thickness_groups(model: &StepModel, bucket_mm: f64) -> Vec<ThicknessInfo> {
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for solid in &model.solids {
        let bucket = thickness_bucket(solid.thickness_mm, bucket_mm);
        *counts.entry(bucket).or_insert(0) += 1;
    }

    let mut groups = counts
        .into_iter()
        .map(|(bucket, count)| ThicknessInfo {
            value_mm: bucket as f64 * bucket_mm,
            count,
        })
        .collect::<Vec<_>>();

    groups.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.value_mm.total_cmp(&b.value_mm))
    });
    groups
}

fn select_thickness(requested: Option<f64>, groups: &[ThicknessInfo]) -> Option<f64> {
    if groups.is_empty() {
        return None;
    }
    if let Some(target) = requested {
        return groups
            .iter()
            .min_by(|a, b| {
                (a.value_mm - target)
                    .abs()
                    .total_cmp(&(b.value_mm - target).abs())
            })
            .map(|g| g.value_mm);
    }
    Some(groups[0].value_mm)
}

fn bbox_center_2d(points: &[Point2D]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
}

fn thickness_bucket(value_mm: f64, bucket_mm: f64) -> i64 {
    let bucket = if bucket_mm > 0.0 { bucket_mm } else { 0.1 };
    (value_mm / bucket).round() as i64
}

#[derive(Debug, Serialize, Deserialize)]
struct StepJsonFile {
    solids: Vec<StepJsonSolid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StepJsonSolid {
    #[serde(default)]
    thickness_mm: f64,
    profile: StepJsonPolygon,
}

#[derive(Debug, Serialize, Deserialize)]
struct StepJsonPolygon {
    outer: Vec<[f64; 2]>,
    #[serde(default)]
    holes: Vec<Vec<[f64; 2]>>,
}

fn parse_step_model(path: &Path, text: &str) -> Result<StepModel, StepImportError> {
    let file: StepJsonFile = serde_json::from_str(text)
        .map_err(|e| StepImportError::ReadFailed(format!("unsupported STEP payload: {e}")))?;

    let mut solids = Vec::with_capacity(file.solids.len());
    for (idx, solid) in file.solids.into_iter().enumerate() {
        let polygon = Polygon {
            outer: solid
                .profile
                .outer
                .into_iter()
                .map(|[x, y]| Point2D { x, y })
                .collect(),
            holes: solid
                .profile
                .holes
                .into_iter()
                .map(|ring| ring.into_iter().map(|[x, y]| Point2D { x, y }).collect())
                .collect(),
        };
        let normalized = normalize_polygon(polygon)
            .map_err(|msg| StepImportError::ReadFailed(format!("solid {idx}: {msg}")))?;
        solids.push(SolidRecord {
            key: StepSolidKey(idx as u32),
            thickness_mm: solid.thickness_mm.max(0.0),
            profile: normalized,
            frame: None,
        });
    }

    Ok(StepModel {
        source_path: path.to_path_buf(),
        solids,
    })
}
