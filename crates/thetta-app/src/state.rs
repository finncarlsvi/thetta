use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use egui::Color32;
use thetta_geometry::types::{NestingResult, Piece, PieceId, SheetConfig};
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::optimizer::run_optimizer;
use thetta_nesting::runner::NestingUpdate;

#[derive(Debug, Clone)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub piece_count: usize,
}

#[derive(Debug, Clone)]
pub struct NestingStats {
    pub utilization_pct: f64,
    pub pieces_placed: usize,
    pub pieces_total: usize,
    pub elapsed: Duration,
    pub iterations: usize,
}

impl Default for NestingStats {
    fn default() -> Self {
        Self {
            utilization_pct: 0.0,
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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_files: vec![],
            pieces: vec![],
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
        }
    }
}

impl AppState {
    pub fn load_json_file(&mut self, path: &Path) -> Result<usize, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let mut parsed: Vec<Piece> =
            serde_json::from_str(&text).map_err(|e| format!("json parse error: {e}"))?;

        for p in &mut parsed {
            p.source_file = path.to_path_buf();
        }

        let added = parsed.len();
        self.pieces.extend(parsed);
        self.loaded_files.push(LoadedFile {
            path: path.to_path_buf(),
            piece_count: added,
        });

        self.assign_file_color(path);
        self.stats.pieces_total = self.pieces.len();
        Ok(added)
    }

    pub fn start_optimizer(&mut self) {
        if self.pieces.is_empty() {
            return;
        }
        self.stop_optimizer();

        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let pieces = self.pieces.clone();
        let sheet = self.sheet_config.clone();
        let config = self.optimizer_config.clone();
        let cancel_bg = cancel.clone();

        thread::spawn(move || {
            run_optimizer(pieces, sheet, config, tx, cancel_bg);
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

    pub fn delete_selected_piece(&mut self) {
        if let Some(pid) = self.selected_piece.take() {
            self.pieces.retain(|p| p.id != pid);
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
        let placed = result.sheets.first().map(|s| s.placements.len()).unwrap_or(0);
        let utilization = result.sheets.first().map(|s| s.utilization).unwrap_or(0.0) * 100.0;
        self.stats.pieces_placed = placed;
        self.stats.pieces_total = self.pieces.len();
        self.stats.utilization_pct = utilization;
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
}
