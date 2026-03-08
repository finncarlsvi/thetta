use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pick_step_file() -> Option<PathBuf> {
    run_osascript(&[
        "set p to POSIX path of (choose file with prompt \"Load STEP\" of type {\"step\",\"stp\"})",
        "return p",
    ])
    .map(PathBuf::from)
}

pub fn pick_export_target(default_name: &str) -> Option<(PathBuf, String)> {
    let script = format!(
        "set p to POSIX path of (choose file name with prompt \"Export STEP\" default name \"{}\")\nreturn p",
        default_name.replace('"', "")
    );
    let path = run_osascript(&[&script])?;
    let full_path = PathBuf::from(path);
    let parent = full_path.parent()?.to_path_buf();
    let stem = file_stem_or_default(&full_path, "nested");
    Some((parent, stem))
}

fn run_osascript(lines: &[&str]) -> Option<String> {
    let mut cmd = Command::new("osascript");
    for line in lines {
        cmd.arg("-e").arg(line);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim().trim_matches('"').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn file_stem_or_default(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}
