use std::path::{Path, PathBuf};
use std::sync::mpsc;

/// Non-blocking file picker that runs on a background thread.
/// Call `request_step_file()` to start, then poll `try_recv()` on the receiver.
pub fn request_step_file() -> mpsc::Receiver<Option<PathBuf>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = rfd::FileDialog::new()
            .set_title("Load STEP")
            .add_filter("STEP files", &["step", "stp"])
            .pick_file();
        let _ = tx.send(result);
    });
    rx
}

/// Non-blocking save dialog. Returns (parent_dir, file_stem).
pub fn request_export_target(default_name: String) -> mpsc::Receiver<Option<(PathBuf, String)>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = rfd::FileDialog::new()
            .set_title("Export STEP")
            .set_file_name(&default_name)
            .add_filter("STEP files", &["step"])
            .save_file()
            .and_then(|path| {
                let parent = path.parent()?.to_path_buf();
                let stem = file_stem_or_default(&path, "nested");
                Some((parent, stem))
            });
        let _ = tx.send(result);
    });
    rx
}

/// Non-blocking folder picker for G-code output.
pub fn request_gcode_directory() -> mpsc::Receiver<Option<PathBuf>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = rfd::FileDialog::new()
            .set_title("Select G-code Output Directory")
            .pick_folder();
        let _ = tx.send(result);
    });
    rx
}

fn file_stem_or_default(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}
