use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use egui::Color32;
use jagua_rs::geometry::geo_enums::RotationRange;
use thetta_geometry::types::{NestingResult, Piece, PieceId, SheetConfig};
use thetta_nesting::benchmark::load_benchmark;
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::optimizer::run_optimizer;
use thetta_nesting::runner::NestingUpdate;
use thetta_step::{
    ExtractionConfig, StepExportError, StepImportError, StepImportResult, StepModel, StepPieceRef,
    ThicknessInfo, export_all_sheets, extract_pieces_for_thickness, import_step,
};

#[derive(Debug, Clone)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub piece_count: usize,
}

#[derive(Debug, Clone)]
pub struct LoadedBenchmark {
    pub name: String,
    pub strip_height: f64,
}

#[derive(Debug, Clone)]
pub struct NestingStats {
    pub utilization_pct: f64,
    pub strip_length: f64,
    pub pieces_placed: usize,
    pub pieces_total: usize,
    pub elapsed: Duration,
    pub iterations: usize,
}

impl Default for NestingStats {
    fn default() -> Self {
        Self {
            utilization_pct: 0.0,
            strip_length: 0.0,
            pieces_placed: 0,
            pieces_total: 0,
            elapsed: Duration::ZERO,
            iterations: 0,
        }
    }
}

pub struct AppState {
    pub loaded_files: Vec<LoadedFile>,
    pub pieces: Vec<Piece>,
    pub piece_orientations: Vec<RotationRange>,
    pub benchmark: Option<LoadedBenchmark>,
    pub sheet_config: SheetConfig,
    pub optimizer_config: OptimizerConfig,
    pub current_result: Option<NestingResult>,
    pub selected_piece: Option<PieceId>,
    pub optimizer_running: bool,
    pub stats: NestingStats,
    pub update_rx: Option<Receiver<NestingUpdate>>,
    pub cancel_token: Option<Arc<AtomicBool>>,
    pub run_started_at: Option<Instant>,
    pub file_colors: HashMap<PathBuf, Color32>,
    pub step_session: Option<Arc<Mutex<StepModel>>>,
    pub step_piece_refs: Vec<StepPieceRef>,
    pub thickness_groups: Vec<ThicknessInfo>,
    pub selected_thickness_idx: usize,
    pub step_loading: bool,
    pub step_error: Option<String>,
    pub step_import_rx: Option<Receiver<Result<StepImportResult, StepImportError>>>,
    pub step_exporting: bool,
    pub step_export_status: Option<String>,
    pub step_export_rx: Option<Receiver<Result<Vec<PathBuf>, StepExportError>>>,
    pub step_source_path: Option<PathBuf>,
    pub step_config: ExtractionConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_files: vec![],
            pieces: vec![],
            piece_orientations: vec![],
            benchmark: None,
            sheet_config: SheetConfig::default(),
            optimizer_config: OptimizerConfig::default(),
            current_result: None,
            selected_piece: None,
            optimizer_running: false,
            stats: NestingStats::default(),
            update_rx: None,
            cancel_token: None,
            run_started_at: None,
            file_colors: HashMap::new(),
            step_session: None,
            step_piece_refs: Vec::new(),
            thickness_groups: Vec::new(),
            selected_thickness_idx: 0,
            step_loading: false,
            step_error: None,
            step_import_rx: None,
            step_exporting: false,
            step_export_status: None,
            step_export_rx: None,
            step_source_path: None,
            step_config: ExtractionConfig::default(),
        }
    }
}

impl AppState {
    pub fn load_json_file(&mut self, path: &Path) -> Result<usize, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("json parse error: {e}"))?;

        let is_benchmark = value.get("name").is_some() && value.get("items").is_some();
        if is_benchmark {
            return self.load_benchmark_file(path);
        }

        let mut parsed: Vec<Piece> =
            serde_json::from_value(value).map_err(|e| format!("json parse error: {e}"))?;

        for p in &mut parsed {
            p.source_file = path.to_path_buf();
        }

        let added = parsed.len();
        self.pieces.extend(parsed);
        self.piece_orientations
            .extend((0..added).map(|_| RotationRange::Continuous));
        self.loaded_files.push(LoadedFile {
            path: path.to_path_buf(),
            piece_count: added,
        });

        self.benchmark = None;
        self.assign_file_color(path);
        self.stats.pieces_total = self.pieces.len();
        self.step_export_status = None;
        Ok(added)
    }

    pub fn load_benchmark_file(&mut self, path: &Path) -> Result<usize, String> {
        let (pieces, sheet, meta) = load_benchmark(path).map_err(|e| e.to_string())?;
        let count = pieces.len();

        self.pieces = pieces;
        self.piece_orientations = meta.orientations;
        self.sheet_config = sheet;
        self.loaded_files = vec![LoadedFile {
            path: path.to_path_buf(),
            piece_count: count,
        }];
        self.benchmark = Some(LoadedBenchmark {
            name: meta.name,
            strip_height: meta.strip_height,
        });
        self.current_result = None;
        self.selected_piece = None;

        self.assign_file_color(path);
        self.stats.pieces_total = count;
        self.clear_step_session();
        Ok(count)
    }

    pub fn start_optimizer(&mut self) {
        if self.pieces.is_empty() {
            return;
        }
        self.stop_optimizer();

        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let pieces = self.pieces.clone();
        let orientations =
            (!self.piece_orientations.is_empty()).then_some(self.piece_orientations.clone());
        let sheet = self.sheet_config.clone();
        let config = self.optimizer_config.clone();
        let cancel_bg = cancel.clone();

        thread::spawn(move || {
            run_optimizer(pieces, sheet, orientations, config, tx, cancel_bg);
        });

        self.update_rx = Some(rx);
        self.cancel_token = Some(cancel);
        self.optimizer_running = true;
        self.run_started_at = Some(Instant::now());
        self.stats.iterations = 0;
    }

    pub fn stop_optimizer(&mut self) {
        if let Some(cancel) = &self.cancel_token {
            cancel.store(true, Ordering::Relaxed);
        }
        self.optimizer_running = false;
        self.cancel_token = None;
        self.update_rx = None;
    }

    pub fn poll_updates(&mut self) {
        let mut finished = false;
        while let Some(rx) = &self.update_rx {
            match rx.try_recv() {
                Ok(NestingUpdate::Improved(result)) => {
                    self.apply_result(result);
                    self.stats.iterations += 1;
                }
                Ok(NestingUpdate::Finished(result)) => {
                    self.apply_result(result);
                    finished = true;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    finished = true;
                    break;
                }
            }
        }

        if let Some(start) = self.run_started_at {
            self.stats.elapsed = start.elapsed();
        }

        if finished {
            self.optimizer_running = false;
            self.update_rx = None;
            self.cancel_token = None;
        }
    }

    pub fn start_step_import(&mut self, path: PathBuf) {
        self.step_loading = true;
        self.step_error = None;
        self.step_export_status = None;
        self.step_import_rx = None;

        let (tx, rx) = mpsc::channel();
        let config = self.step_config;
        thread::spawn(move || {
            let result = import_step(&path, None, &config);
            let _ = tx.send(result);
        });
        self.step_import_rx = Some(rx);
    }

    pub fn poll_step_import(&mut self) {
        let Some(rx) = &self.step_import_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(result)) => {
                self.step_loading = false;
                self.step_import_rx = None;
                self.apply_step_import(result);
                self.start_optimizer();
            }
            Ok(Err(err)) => {
                self.step_loading = false;
                self.step_import_rx = None;
                self.step_error = Some(err.to_string());
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.step_loading = false;
                self.step_import_rx = None;
                self.step_error = Some("STEP import thread disconnected".to_string());
            }
        }
    }

    pub fn select_thickness(&mut self, idx: usize) {
        if idx >= self.thickness_groups.len() {
            return;
        }
        let Some(session) = &self.step_session else {
            return;
        };

        let selected = self.thickness_groups[idx].value_mm;
        let (pieces, refs) = {
            let lock = session.lock();
            let Ok(model) = lock else {
                self.step_error = Some("failed to lock STEP model".to_string());
                return;
            };
            match extract_pieces_for_thickness(&model, selected, &self.step_config) {
                Ok(data) => data,
                Err(err) => {
                    self.step_error = Some(err.to_string());
                    return;
                }
            }
        };

        self.selected_thickness_idx = idx;
        self.step_error = None;
        self.replace_step_pieces(pieces, refs);
        self.start_optimizer();
    }

    pub fn start_step_export(&mut self, out_dir: PathBuf, file_stem: String) {
        let (Some(session), Some(result)) =
            (self.step_session.clone(), self.current_result.clone())
        else {
            self.step_error = Some("nothing to export".to_string());
            return;
        };
        let refs = self.step_piece_refs.clone();
        let (tx, rx) = mpsc::channel();

        self.step_exporting = true;
        self.step_export_status = None;
        self.step_export_rx = Some(rx);

        thread::spawn(move || {
            let model = match session.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => {
                    let _ = tx.send(Err(StepExportError::WriteError(
                        "failed to lock STEP model".to_string(),
                    )));
                    return;
                }
            };

            let exported = export_all_sheets(&model, &refs, &result, &out_dir, &file_stem);
            let _ = tx.send(exported);
        });
    }

    pub fn poll_step_export(&mut self) {
        let Some(rx) = &self.step_export_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(paths)) => {
                self.step_exporting = false;
                self.step_export_rx = None;

                let mut non_step = 0usize;
                if let Some(result) = &self.current_result {
                    let mapped = self
                        .step_piece_refs
                        .iter()
                        .map(|r| r.piece_id)
                        .collect::<std::collections::HashSet<_>>();
                    for sheet in &result.sheets {
                        non_step += sheet
                            .placements
                            .iter()
                            .filter(|p| !mapped.contains(&p.piece_id))
                            .count();
                    }
                }

                self.step_export_status = Some(if non_step == 0 {
                    format!("Exported {} sheets", paths.len())
                } else {
                    format!(
                        "Exported {} sheets (skipped {non_step} non-STEP placements)",
                        paths.len()
                    )
                });
            }
            Ok(Err(err)) => {
                self.step_exporting = false;
                self.step_export_rx = None;
                self.step_error = Some(err.to_string());
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.step_exporting = false;
                self.step_export_rx = None;
                self.step_error = Some("STEP export thread disconnected".to_string());
            }
        }
    }

    pub fn delete_selected_piece(&mut self) {
        if let Some(pid) = self.selected_piece.take()
            && let Some(idx) = self.pieces.iter().position(|p| p.id == pid)
        {
            self.pieces.remove(idx);
            if idx < self.piece_orientations.len() {
                self.piece_orientations.remove(idx);
            }
            self.stats.pieces_total = self.pieces.len();
            self.start_optimizer();
        }
    }

    pub fn piece_color(&self, path: &Path) -> Color32 {
        self.file_colors
            .get(path)
            .copied()
            .unwrap_or(Color32::LIGHT_BLUE)
    }

    fn apply_result(&mut self, result: NestingResult) {
        let placed = result
            .sheets
            .first()
            .map(|s| s.placements.len())
            .unwrap_or(0);
        let utilization = result.sheets.first().map(|s| s.utilization).unwrap_or(0.0) * 100.0;
        let strip_length = result
            .sheets
            .first()
            .map(|s| s.strip_length)
            .unwrap_or(self.sheet_config.width);

        self.stats.pieces_placed = placed;
        self.stats.pieces_total = self.pieces.len();
        self.stats.utilization_pct = utilization;
        self.stats.strip_length = strip_length;
        self.current_result = Some(result);
    }

    fn assign_file_color(&mut self, path: &Path) {
        if self.file_colors.contains_key(path) {
            return;
        }
        let palette = [
            Color32::from_rgb(0xE6, 0x7E, 0x22),
            Color32::from_rgb(0x2E, 0x86, 0xAB),
            Color32::from_rgb(0x2A, 0x9D, 0x8F),
            Color32::from_rgb(0xBC, 0x47, 0x49),
            Color32::from_rgb(0x8D, 0x6E, 0x63),
            Color32::from_rgb(0x3A, 0x86, 0xFF),
        ];
        let idx = self.file_colors.len() % palette.len();
        self.file_colors.insert(path.to_path_buf(), palette[idx]);
    }

    fn apply_step_import(&mut self, result: StepImportResult) {
        let path = result.model.source_path().to_path_buf();
        self.step_source_path = Some(path.clone());
        self.thickness_groups = result.thicknesses;
        self.selected_thickness_idx = self
            .thickness_groups
            .iter()
            .position(|g| (g.value_mm - result.selected_thickness_mm).abs() < 1e-9)
            .unwrap_or(0);
        self.step_session = Some(Arc::new(Mutex::new(result.model)));
        self.step_error = None;
        self.replace_step_pieces(result.pieces, result.piece_refs);
    }

    fn replace_step_pieces(&mut self, new_pieces: Vec<Piece>, new_refs: Vec<StepPieceRef>) {
        let old_step_ids = self
            .step_piece_refs
            .iter()
            .map(|r| r.piece_id)
            .collect::<std::collections::HashSet<_>>();

        let mut retained = Vec::new();
        let mut retained_orientations = Vec::new();
        for (idx, piece) in self.pieces.iter().enumerate() {
            if !old_step_ids.contains(&piece.id) {
                retained.push(piece.clone());
                if let Some(orientation) = self.piece_orientations.get(idx) {
                    retained_orientations.push(orientation.clone());
                }
            }
        }

        let step_count = new_pieces.len();
        retained.extend(new_pieces);
        retained_orientations.extend((0..step_count).map(|_| RotationRange::Continuous));

        self.pieces = retained;
        self.piece_orientations = retained_orientations;
        self.step_piece_refs = new_refs;
        self.stats.pieces_total = self.pieces.len();

        if let Some(path) = self.step_source_path.clone() {
            self.assign_file_color(&path);
            self.loaded_files.retain(|lf| lf.path != path);
            self.loaded_files.push(LoadedFile {
                path,
                piece_count: step_count,
            });
        }
    }

    fn clear_step_session(&mut self) {
        self.step_session = None;
        self.step_piece_refs.clear();
        self.thickness_groups.clear();
        self.selected_thickness_idx = 0;
        self.step_loading = false;
        self.step_error = None;
        self.step_import_rx = None;
        self.step_exporting = false;
        self.step_export_rx = None;
        self.step_export_status = None;
        self.step_source_path = None;
    }
}
