use std::io;
use std::path::{Path, PathBuf};

use thetta_geometry::types::ToolTable;

pub fn tool_db_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("THETTA_TOOL_DB_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    dirs::config_dir().map(|dir| dir.join("thetta").join("tools.json"))
}

pub fn load_tool_table() -> ToolTable {
    let Some(path) = tool_db_path() else {
        return ToolTable::default();
    };
    load_tool_table_from_path(&path).unwrap_or_default()
}

pub fn load_tool_table_from_path(path: &Path) -> Result<ToolTable, io::Error> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(serde_json::from_str(&contents).unwrap_or_default()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(ToolTable::default()),
        Err(err) => Err(err),
    }
}

pub fn save_tool_table(table: &ToolTable) -> Result<(), io::Error> {
    let Some(path) = tool_db_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no config directory available",
        ));
    };
    save_tool_table_to_path(table, &path)
}

pub fn save_tool_table_to_path(table: &ToolTable, path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(table)
        .map_err(|err| io::Error::other(format!("failed to serialize tool table: {err}")))?;

    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::types::Tool;

    fn sample_table() -> ToolTable {
        ToolTable {
            tools: vec![Tool {
                number: 1,
                name: "6mm 2-flute upcut".to_string(),
                diameter: 6.0,
                flute_count: 2,
                feed_rate: 1500.0,
                plunge_rate: 500.0,
                spindle_speed: 18000,
            }],
        }
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tools.json");
        let table = sample_table();

        save_tool_table_to_path(&table, &path).unwrap();
        let loaded = load_tool_table_from_path(&path).unwrap();

        assert_eq!(loaded, table);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.json");
        let loaded = load_tool_table_from_path(&path).unwrap();
        assert!(loaded.tools.is_empty());
    }

    #[test]
    fn load_corrupt_json_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tools.json");
        std::fs::write(&path, "{bad json").unwrap();

        let loaded = load_tool_table_from_path(&path).unwrap();
        assert!(loaded.tools.is_empty());
    }
}
