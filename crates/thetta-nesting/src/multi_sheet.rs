use crate::conversion::default_orientations;
use crate::engine::NestingEngine;
use jagua_rs::geometry::geo_enums::RotationRange;
use std::collections::HashMap;
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{NestingResult, Piece, PieceId, SheetConfig};

/// Orchestrates multiple independent SPP engines (one per sheet).
#[derive(Clone)]
pub struct MultiSheetEngine {
    sheets: Vec<NestingEngine>,
    all_pieces: Vec<Piece>,
    sheet_config: SheetConfig,
    orientations: Vec<RotationRange>,
    /// piece_idx -> Some(sheet_idx) for placed pieces.
    assignment: Vec<Option<usize>>,
    piece_idx_by_id: HashMap<PieceId, usize>,
}

impl MultiSheetEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig, orientations: &[RotationRange]) -> Self {
        let mut piece_idx_by_id = HashMap::with_capacity(pieces.len());
        for (idx, piece) in pieces.iter().enumerate() {
            piece_idx_by_id.insert(piece.id, idx);
        }

        Self {
            sheets: Vec::new(),
            all_pieces: pieces.to_vec(),
            sheet_config: sheet.clone(),
            orientations: if orientations.len() == pieces.len() {
                orientations.to_vec()
            } else {
                default_orientations(pieces.len())
            },
            assignment: vec![None; pieces.len()],
            piece_idx_by_id,
        }
    }

    pub fn piece_count(&self) -> usize {
        self.all_pieces.len()
    }

    pub fn piece_area(&self, idx: usize) -> f64 {
        self.all_pieces
            .get(idx)
            .map(|p| polygon_ops::area(&p.polygon))
            .unwrap_or(0.0)
    }

    pub fn rotation_range(&self, idx: usize) -> Option<&RotationRange> {
        self.orientations.get(idx)
    }

    pub fn open_sheet(&mut self) -> usize {
        let engine = NestingEngine::new_with_orientations(
            &self.all_pieces,
            &self.sheet_config,
            Some(&self.orientations),
        );
        self.sheets.push(engine);
        self.sheets.len() - 1
    }

    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }

    pub fn sheet_engine(&self, sheet_idx: usize) -> &NestingEngine {
        &self.sheets[sheet_idx]
    }

    pub fn sheet_engine_mut(&mut self, sheet_idx: usize) -> &mut NestingEngine {
        &mut self.sheets[sheet_idx]
    }

    pub fn pieces_on_sheet(&self, sheet_idx: usize) -> Vec<usize> {
        self.assignment
            .iter()
            .enumerate()
            .filter_map(|(idx, assigned)| (*assigned == Some(sheet_idx)).then_some(idx))
            .collect()
    }

    pub fn assignment_of(&self, piece_idx: usize) -> Option<usize> {
        self.assignment.get(piece_idx).copied().flatten()
    }

    pub fn place_on_sheet(
        &mut self,
        piece_idx: usize,
        sheet_idx: usize,
        x: f64,
        y: f64,
        rotation: f64,
    ) -> bool {
        if piece_idx >= self.all_pieces.len() || sheet_idx >= self.sheets.len() {
            return false;
        }

        let previous_sheet = self.assignment[piece_idx];
        if self.sheets[sheet_idx].place_item(piece_idx, x, y, rotation) {
            if let Some(prev_sheet_idx) = previous_sheet
                && prev_sheet_idx != sheet_idx
                && prev_sheet_idx < self.sheets.len()
            {
                self.sheets[prev_sheet_idx].remove_item(piece_idx);
            }
            self.assignment[piece_idx] = Some(sheet_idx);
            true
        } else {
            false
        }
    }

    /// Place a piece on a sheet without collision checks (for writing back
    /// validated GLS/compression results).
    pub fn force_place_on_sheet(
        &mut self,
        piece_idx: usize,
        sheet_idx: usize,
        x: f64,
        y: f64,
        rotation: f64,
    ) {
        if piece_idx >= self.all_pieces.len() || sheet_idx >= self.sheets.len() {
            return;
        }

        if let Some(prev_sheet_idx) = self.assignment[piece_idx] {
            if prev_sheet_idx != sheet_idx && prev_sheet_idx < self.sheets.len() {
                self.sheets[prev_sheet_idx].remove_item(piece_idx);
            }
        }
        self.sheets[sheet_idx].force_place_item(piece_idx, x, y, rotation);
        self.assignment[piece_idx] = Some(sheet_idx);
    }

    pub fn remove_piece(&mut self, piece_idx: usize) {
        if piece_idx >= self.assignment.len() {
            return;
        }

        if let Some(sheet_idx) = self.assignment[piece_idx].take()
            && sheet_idx < self.sheets.len()
        {
            self.sheets[sheet_idx].remove_item(piece_idx);
        }
    }

    pub fn current_result(&self) -> NestingResult {
        let mut sheets = Vec::new();
        for (sheet_idx, engine) in self.sheets.iter().enumerate() {
            let mut sheet = engine
                .current_result()
                .sheets
                .into_iter()
                .next()
                .unwrap_or_default();
            sheet
                .placements
                .retain(|p| self.piece_sheet_idx(p.piece_id) == Some(sheet_idx));
            // In multi-sheet mode, clamp strip_length to sheet width
            if sheet.strip_length > self.sheet_config.width {
                sheet.strip_length = self.sheet_config.width;
            }
            if !sheet.placements.is_empty() {
                sheets.push(sheet);
            }
        }

        let unplaced = self
            .all_pieces
            .iter()
            .enumerate()
            .filter_map(|(idx, piece)| self.assignment[idx].is_none().then_some(piece.id))
            .collect();

        NestingResult { sheets, unplaced }
    }

    fn piece_sheet_idx(&self, piece_id: PieceId) -> Option<usize> {
        let idx = *self.piece_idx_by_id.get(&piece_id)?;
        self.assignment.get(idx).copied().flatten()
    }
}
