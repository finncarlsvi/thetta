use thetta_geometry::types::Piece;
use std::path::Path;

/// Import a STEP file and extract 2D profiles for a given thickness.
/// Returns pieces and a list of all detected thicknesses with counts.
pub fn import_step(
    _path: &Path,
    _thickness: Option<f64>,
    _tolerance: f64,
) -> Result<StepImportResult, StepImportError> {
    todo!("OpenCASCADE FFI integration")
}

pub struct StepImportResult {
    pub pieces: Vec<Piece>,
    pub thicknesses: Vec<ThicknessInfo>,
    pub selected_thickness: f64,
}

pub struct ThicknessInfo {
    pub value: f64,
    pub count: usize,
}

#[derive(Debug)]
pub enum StepImportError {
    FileNotFound,
    ParseError(String),
    NoSolidsFound,
}
