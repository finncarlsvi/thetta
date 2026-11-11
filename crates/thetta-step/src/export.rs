use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use serde::Serialize;
use thetta_geometry::types::{NestingResult, Sheet, Tab};

use crate::import::{StepModel, StepPieceRef, StepSolidKey};
use crate::profile::transform_polygon;

#[derive(Debug, Clone)]
pub enum StepExportError {
    InvalidOutputPath(String),
    MissingSolidForPiece(u64),
    WriteError(String),
    OcctError(String),
}

impl Display for StepExportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOutputPath(msg) => write!(f, "invalid output path: {msg}"),
            Self::MissingSolidForPiece(id) => write!(f, "missing solid mapping for piece {id}"),
            Self::WriteError(msg) => write!(f, "write error: {msg}"),
            Self::OcctError(msg) => write!(f, "occt error: {msg}"),
        }
    }
}

impl std::error::Error for StepExportError {}

pub fn export_sheet(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    sheet_config: &thetta_geometry::types::SheetConfig,
    output_path: &Path,
    tabs: Option<&HashMap<(usize, usize), Vec<Tab>>>,
    sheet_index: usize,
) -> Result<(), StepExportError> {
    let parent = output_path.parent().ok_or_else(|| {
        StepExportError::InvalidOutputPath(format!(
            "{} has no parent directory",
            output_path.display()
        ))
    })?;
    if !parent.exists() {
        return Err(StepExportError::InvalidOutputPath(format!(
            "{} does not exist",
            parent.display()
        )));
    }

    // Use real OCCT export if available and source is a STEP file
    #[cfg(feature = "occt")]
    {
        let is_step = model
            .source_path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("step") || ext.eq_ignore_ascii_case("stp"));
        if is_step {
            return export_sheet_occt(model, piece_refs, sheet, sheet_config, output_path);
        }
    }

    // Fallback: JSON-based export
    export_sheet_json(model, piece_refs, sheet, output_path, tabs, sheet_index)
}

fn export_sheet_json(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    output_path: &Path,
    tabs: Option<&HashMap<(usize, usize), Vec<Tab>>>,
    sheet_index: usize,
) -> Result<(), StepExportError> {
    let key_by_piece = build_piece_map(piece_refs);
    let mut solids = Vec::new();
    for placement in &sheet.placements {
        let Some(solid_key) = key_by_piece.get(&placement.piece_id) else {
            continue;
        };
        let Some(profile) = model.solid_profile(*solid_key) else {
            return Err(StepExportError::MissingSolidForPiece(placement.piece_id.0));
        };
        let transformed = transform_polygon(profile, placement.x, placement.y, placement.rotation);
        let thickness = piece_refs
            .iter()
            .find(|r| r.piece_id == placement.piece_id)
            .map(|r| r.thickness_mm)
            .unwrap_or(0.0);
        solids.push(ExportSolid::from_polygon(&transformed, thickness));
    }

    let sheet_tabs = tabs.and_then(|tab_map| {
        let placement_tabs = sheet
            .placements
            .iter()
            .enumerate()
            .filter_map(|(placement_idx, placement)| {
                tab_map
                    .get(&(sheet_index, placement_idx))
                    .map(|tabs| ExportPlacementTabs {
                        piece_id: placement.piece_id.0,
                        placement_index: placement_idx,
                        tabs: tabs
                            .iter()
                            .map(|t| ExportTab {
                                x: t.position.x,
                                y: t.position.y,
                                edge_index: t.edge_index,
                                t: t.t,
                                width: t.width,
                                height: t.height,
                            })
                            .collect(),
                    })
            })
            .collect::<Vec<_>>();
        (!placement_tabs.is_empty()).then_some(placement_tabs)
    });

    let json = serde_json::to_string_pretty(&ExportFile {
        solids,
        tabs: sheet_tabs,
    })
    .map_err(|e| StepExportError::WriteError(e.to_string()))?;
    std::fs::write(output_path, json).map_err(|e| StepExportError::WriteError(e.to_string()))?;
    Ok(())
}

#[cfg(feature = "occt")]
fn export_sheet_occt(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    sheet_config: &thetta_geometry::types::SheetConfig,
    output_path: &Path,
) -> Result<(), StepExportError> {
    use crate::occt;

    // Re-read the source STEP file to get original 3D solids
    let reader = occt::OcctStepReader::read_file(model.source_path())
        .map_err(|e| StepExportError::OcctError(format!("re-reading source: {e}")))?;
    let solid_infos = occt::enumerate_solids(reader.root_shape());

    let key_by_piece = build_piece_map(piece_refs);
    let mut writer = occt::OcctStepWriter::new();

    // Add CNC bed panel (extruded 10mm downward from z=0)
    let bed_width = sheet.strip_length.max(sheet_config.width);
    let bed = occt::make_box(bed_width, sheet_config.height, 10.0);
    writer.add_shape(&bed)?;

    for placement in &sheet.placements {
        let Some(solid_key) = key_by_piece.get(&placement.piece_id) else {
            continue;
        };
        let solid_idx = solid_key.0 as usize;
        if solid_idx >= solid_infos.len() {
            return Err(StepExportError::MissingSolidForPiece(placement.piece_id.0));
        }

        let frame = model.solid_frame(*solid_key).ok_or_else(|| {
            StepExportError::OcctError(format!(
                "missing profile frame for piece {}",
                placement.piece_id.0
            ))
        })?;

        let transformed = occt::transform_shape(
            &solid_infos[solid_idx].solid,
            placement.x,
            placement.y,
            placement.rotation,
            &frame.frame_3d,
            frame.center_2d,
        );
        writer.add_shape(&transformed)?;
    }

    writer.write(output_path)?;
    Ok(())
}

pub fn export_all_sheets(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    result: &NestingResult,
    sheet_config: &thetta_geometry::types::SheetConfig,
    out_dir: &Path,
    file_stem: &str,
    tabs: Option<&HashMap<(usize, usize), Vec<Tab>>>,
) -> Result<Vec<PathBuf>, StepExportError> {
    if !out_dir.exists() || !out_dir.is_dir() {
        return Err(StepExportError::InvalidOutputPath(format!(
            "{} is not a directory",
            out_dir.display()
        )));
    }
    if file_stem.trim().is_empty() {
        return Err(StepExportError::InvalidOutputPath(
            "file stem must not be empty".to_string(),
        ));
    }

    let ext = model
        .source_path()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("step");

    let mut written = Vec::with_capacity(result.sheets.len());
    for (idx, sheet) in result.sheets.iter().enumerate() {
        let path = out_dir.join(format!("{file_stem}_sheet_{:03}.{ext}", idx + 1));
        export_sheet(model, piece_refs, sheet, sheet_config, &path, tabs, idx)?;
        written.push(path);
    }
    Ok(written)
}

fn build_piece_map(
    piece_refs: &[StepPieceRef],
) -> HashMap<thetta_geometry::types::PieceId, StepSolidKey> {
    piece_refs
        .iter()
        .map(|r| (r.piece_id, r.solid_key))
        .collect::<HashMap<_, _>>()
}

#[derive(Debug, Serialize)]
struct ExportFile {
    solids: Vec<ExportSolid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tabs: Option<Vec<ExportPlacementTabs>>,
}

#[derive(Debug, Serialize)]
struct ExportSolid {
    thickness_mm: f64,
    profile: ExportPolygon,
}

#[derive(Debug, Serialize)]
struct ExportPolygon {
    outer: Vec<[f64; 2]>,
    holes: Vec<Vec<[f64; 2]>>,
}

#[derive(Debug, Serialize)]
struct ExportPlacementTabs {
    piece_id: u64,
    placement_index: usize,
    tabs: Vec<ExportTab>,
}

#[derive(Debug, Serialize)]
struct ExportTab {
    x: f64,
    y: f64,
    edge_index: usize,
    t: f64,
    width: f64,
    height: f64,
}

impl ExportSolid {
    fn from_polygon(polygon: &thetta_geometry::types::Polygon, thickness_mm: f64) -> Self {
        Self {
            thickness_mm,
            profile: ExportPolygon {
                outer: polygon.outer.iter().map(|p| [p.x, p.y]).collect(),
                holes: polygon
                    .holes
                    .iter()
                    .map(|ring| ring.iter().map(|p| [p.x, p.y]).collect())
                    .collect(),
            },
        }
    }
}
