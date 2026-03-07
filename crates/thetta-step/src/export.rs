use thetta_geometry::types::Sheet;
use std::path::Path;

/// Export a single sheet's placements as a STEP file.
pub fn export_sheet(
    _sheet: &Sheet,
    _output_path: &Path,
) -> Result<(), StepExportError> {
    todo!("OpenCASCADE FFI integration")
}

#[derive(Debug)]
pub enum StepExportError {
    WriteError(String),
}
