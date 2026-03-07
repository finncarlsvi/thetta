use crate::conversion::{
    default_orientations, pieces_to_sp_instance, to_external_transformation, to_internal_transformation,
};
use crate::hazard_counter::CountingCollector;
use jagua_rs::entities::{Instance, PItemKey};
use jagua_rs::geometry::geo_enums::RotationRange;
use jagua_rs::geometry::geo_traits::Transformable;
use jagua_rs::probs::spp::entities::{SPPlacement, SPProblem};
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{NestingResult, Piece, Placement, Sheet, SheetConfig};

#[derive(Clone)]
pub struct NestingEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    pub instance: jagua_rs::probs::spp::entities::SPInstance,
    pub problem: SPProblem,
    pk_map: Vec<Option<PItemKey>>,
    orientations: Vec<RotationRange>,
}

impl NestingEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig) -> Self {
        let orientations = default_orientations(pieces.len());
        Self::new_with_orientations(pieces, sheet, Some(&orientations))
    }

    pub fn new_with_orientations(
        pieces: &[Piece],
        sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self {
        let per_piece_rad = orientations.map(|ranges| {
            ranges
                .iter()
                .map(|range| match range {
                    RotationRange::None => vec![0.0],
                    RotationRange::Continuous => Vec::new(),
                    RotationRange::Discrete(angles) => {
                        angles.iter().map(|a| *a as f64).collect::<Vec<_>>()
                    }
                })
                .collect::<Vec<_>>()
        });

        let instance = pieces_to_sp_instance(pieces, sheet, per_piece_rad.as_deref())
            .expect("failed to convert pieces to SPInstance");

        let mut problem = SPProblem::new(instance.clone());
        if sheet.width.is_finite() && sheet.width > 0.0 {
            problem.change_strip_width(sheet.width as f32);
        }

        Self {
            pieces: pieces.to_vec(),
            sheet: sheet.clone(),
            instance,
            problem,
            pk_map: vec![None; pieces.len()],
            orientations: orientations
                .map(|o| o.to_vec())
                .unwrap_or_else(|| default_orientations(pieces.len())),
        }
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn piece_area(&self, idx: usize) -> f64 {
        self.pieces
            .get(idx)
            .map(|p| polygon_ops::area(&p.polygon))
            .unwrap_or(0.0)
    }

    pub fn sheet_config(&self) -> &SheetConfig {
        &self.sheet
    }

    pub fn rotation_range(&self, idx: usize) -> Option<&RotationRange> {
        self.orientations.get(idx)
    }

    pub fn place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) -> bool {
        if idx >= self.pieces.len() {
            return false;
        }
        if self.collision_cost(idx, x, y, rotation) > 0.0 {
            return false;
        }

        if self.pk_map[idx].is_some() {
            self.remove_item(idx);
        }

        let d_transf = to_internal_transformation(&self.instance, idx, x, y, rotation);
        let pk = self.problem.place_item(SPPlacement {
            item_id: idx,
            d_transf,
        });
        self.pk_map[idx] = Some(pk);
        true
    }

    pub fn force_place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) {
        if idx >= self.pieces.len() {
            return;
        }

        if self.pk_map[idx].is_some() {
            self.remove_item(idx);
        }

        let d_transf = to_internal_transformation(&self.instance, idx, x, y, rotation);
        let pk = self.problem.place_item(SPPlacement {
            item_id: idx,
            d_transf,
        });
        self.pk_map[idx] = Some(pk);
    }

    pub fn remove_item(&mut self, idx: usize) {
        if idx >= self.pieces.len() {
            return;
        }

        if let Some(pk) = self.pk_map[idx].take() {
            let _ = self.problem.remove_item(pk);
        }
    }

    pub fn placement_of(&self, idx: usize) -> Option<Placement> {
        if idx >= self.pieces.len() {
            return None;
        }
        let pk = self.pk_map[idx]?;
        let pi = self.problem.layout.placed_items.get(&pk)?;
        let ext = to_external_transformation(&self.instance, idx, &pi.d_transf);
        let (x, y) = ext.translation();

        Some(Placement {
            piece_id: self.pieces[idx].id,
            x: x as f64,
            y: y as f64,
            rotation: ext.rotation() as f64,
        })
    }

    pub fn collision_cost(&self, idx: usize, x: f64, y: f64, rotation: f64) -> f64 {
        if idx >= self.pieces.len() {
            return f64::INFINITY;
        }

        let item = self.instance.item(idx);
        let d_transf = to_internal_transformation(&self.instance, idx, x, y, rotation);
        let transformed = item.shape_cd.transform_clone(&d_transf.compose());

        let own_hkey = self.pk_map[idx]
            .and_then(|pk| self.problem.layout.cde().haz_key_from_pi_key(pk));
        let mut collector = CountingCollector::new(own_hkey);
        self.problem
            .layout
            .cde()
            .collect_poly_collisions(&transformed, &mut collector);

        collector.count() as f64
    }

    pub fn current_result(&self) -> NestingResult {
        let placements = (0..self.pieces.len())
            .filter_map(|idx| self.placement_of(idx))
            .collect::<Vec<_>>();

        let unplaced = self
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| self.pk_map[idx].is_none().then_some(p.id))
            .collect::<Vec<_>>();

        NestingResult {
            sheets: vec![Sheet {
                placements,
                utilization: self.utilization(),
                strip_length: self.strip_width(),
            }],
            unplaced,
        }
    }

    pub fn utilization(&self) -> f64 {
        let used_area = self
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| self.pk_map[idx].is_some().then_some(polygon_ops::area(&p.polygon)))
            .sum::<f64>();

        let total_area = self.strip_width() * self.sheet.height;
        if total_area <= f64::EPSILON {
            0.0
        } else {
            used_area / total_area
        }
    }

    pub fn strip_width(&self) -> f64 {
        self.problem.strip_width() as f64
    }

    pub fn fit_strip(&mut self) {
        if self.problem.layout.placed_items.is_empty() {
            return;
        }
        self.problem.fit_strip();
    }

    pub fn change_strip_width(&mut self, width: f64) {
        if width.is_finite() && width > 0.0 {
            self.problem.change_strip_width(width as f32);
        }
    }

    pub fn clear_placements(&mut self) {
        for idx in 0..self.pk_map.len() {
            self.remove_item(idx);
        }
    }
}
