use std::path::{Path, PathBuf};

pub fn pick_step_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Load STEP")
        .add_filter("STEP files", &["step", "stp"])
        .pick_file()
}

pub fn pick_export_target(default_name: &str) -> Option<(PathBuf, String)> {
    let path = rfd::FileDialog::new()
        .set_title("Export STEP")
        .set_file_name(default_name)
        .add_filter("STEP files", &["step"])
        .save_file()?;

    let parent = path.parent()?.to_path_buf();
    let stem = file_stem_or_default(&path, "nested");
    Some((parent, stem))
}

fn file_stem_or_default(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}
