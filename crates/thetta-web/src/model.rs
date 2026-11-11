use jagua_rs::geometry::geo_enums::RotationRange;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use thetta_geometry::{polygon_ops, tabs, types::*};
use thetta_nesting::gls::OptimizerConfig;
use thetta_step::{StepModel, StepPieceRef};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub cam: crate::cam::CamSettings,
    pub sheet: SheetConfig,
    pub optimizer: OptimizerConfig,
    pub mode: String,
    pub tabs: TabConfig,
    pub tools: ToolTable,
    pub gcode: GcodeConfig,
    pub profile: ProfileCutConfig,
    pub drill: DrillConfig,
    pub pocket: PocketConfig,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            cam: Default::default(),
            sheet: SheetConfig::default(),
            optimizer: OptimizerConfig::default(),
            mode: "sheet".into(),
            tabs: TabConfig::default(),
            tools: ToolTable::default(),
            gcode: GcodeConfig::default(),
            profile: ProfileCutConfig::default(),
            drill: DrillConfig::default(),
            pocket: PocketConfig::default(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.cam.material.is_empty() && !crate::cam::valid_material(&self.cam.material) {
            return Err("Unsupported project material".into());
        }
        if self
            .cam
            .part_materials
            .values()
            .any(|m| !crate::cam::valid_material(m))
        {
            return Err("Unsupported part material".into());
        }
        for o in self.cam.overrides.values() {
            for value in [o.feed, o.ramp_feed, o.step_down, o.step_over]
                .into_iter()
                .flatten()
            {
                if !value.is_finite() || value <= 0. {
                    return Err("Cutting overrides must be positive".into());
                }
            }
        }
        if self.cam.part_materials.len() > 2000
            || self
                .cam
                .part_materials
                .keys()
                .any(|k| k.parse::<u64>().is_err())
        {
            return Err("Invalid part material assignments".into());
        }
        let s = &self.sheet;
        if !(0.01..=100000.).contains(&s.width)
            || !(0.01..=100000.).contains(&s.height)
            || !(0.0..=s.width.min(s.height)).contains(&s.separation)
        {
            return Err("Invalid sheet dimensions or spacing".into());
        }
        if !["sheet", "strip"].contains(&self.mode.as_str()) {
            return Err("Invalid nesting mode".into());
        }
        let o = &self.optimizer;
        if !(0.1..=3600.).contains(&o.time_limit_secs)
            || !(1..=10000).contains(&o.samples_per_move)
            || o.refinement_depth > 20
            || !(0.001..=1000.).contains(&o.compression_step)
            || !(0.001..=100.).contains(&o.penalty_increment)
        {
            return Err("Invalid optimizer settings".into());
        }
        let t = &self.tabs;
        if t.min_tabs > t.max_tabs
            || t.max_tabs > 100
            || !(1.0..=100000.).contains(&t.tab_distance)
            || [t.tab_width, t.tab_height, t.material_thickness]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.)
            || [t.size_threshold, t.corner_exclusion]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.)
        {
            return Err("Invalid tab settings".into());
        }
        self.tools.validate_definitions()?;
        // Tools may remain unassigned while editing. Full validation happens at export.
        let mut g = self.gcode.clone();
        g.tool_number = Some(1);
        g.validate()?;
        let mut d = self.drill.clone();
        d.enabled = false;
        d.validate()?;
        let mut p = self.pocket.clone();
        p.enabled = false;
        p.validate()?;
        if !["G54", "G55", "G56", "G57", "G58", "G59"]
            .contains(&self.gcode.coordinate_system.as_str())
        {
            return Err("Choose G54–G59".into());
        }
        if !(0.0..=1000.).contains(&self.profile.lead_in_radius) {
            return Err("Invalid lead-in radius".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Part {
    pub piece: Piece,
    pub enabled: bool,
    pub rotation: String,
    #[serde(default)]
    pub thickness_mm: Option<f64>,
}
pub const MAX_THICKNESS_MM: f64 = 30.0;
pub const THICKNESS_BUCKET_MM: f64 = 0.25;
pub fn thickness_key(value: f64) -> i64 {
    ((value / THICKNESS_BUCKET_MM).round() as i64).max(1)
}
impl Part {
    pub fn eligible(&self) -> bool {
        self.thickness_mm
            .is_some_and(|v| v.is_finite() && v > 0.0 && v <= MAX_THICKNESS_MM + 1e-6)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct TabSet {
    pub sheet: usize,
    pub placement: usize,
    pub tabs: Vec<Tab>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct OnshapeSource {
    pub url: String,
    pub name: String,
    pub imported_at: i64,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    pub filename: Option<String>,
    pub source: Option<String>,
    pub thickness: Option<f64>,
    pub groups: Vec<(f64, usize)>,
    pub parts: Vec<Part>,
    pub settings: Settings,
    pub result: Option<NestingResult>,
    pub sheet_thicknesses: Vec<f64>,
    pub sheet_materials: Vec<String>,
    pub cam_plan: Option<crate::cam::Plan>,
    pub excluded_thicknesses: Vec<f64>,
    pub tabs: Vec<TabSet>,
    pub revision: u64,
    pub onshape: Option<OnshapeSource>,
}
impl Saved {
    pub fn includes(&self, part: &Part) -> bool {
        part.eligible()
            && !self.settings.cam.excluded_groups.iter().any(|key| {
                part.thickness_mm.is_some_and(|t| {
                    *key == format!(
                        "{}:{}",
                        crate::cam::material_for(self, part.piece.id.0),
                        thickness_key(t) as f64 * THICKNESS_BUCKET_MM
                    )
                })
            })
            && part.thickness_mm.is_some_and(|value| {
                !self
                    .excluded_thicknesses
                    .iter()
                    .any(|excluded| thickness_key(*excluded) == thickness_key(value))
            })
    }
    pub fn panel_depth(&self, sheet: &Sheet) -> Option<f64> {
        sheet
            .placements
            .iter()
            .filter_map(|p| {
                let parent = PieceId(p.piece_id.0.checked_sub(1)? / 1001);
                self.parts
                    .iter()
                    .find(|part| part.piece.id == parent)
                    .and_then(|part| part.thickness_mm)
            })
            .reduce(f64::max)
    }
}
pub struct Workspace {
    pub dir: PathBuf,
    pub saved: Saved,
    pub model: Option<StepModel>,
    pub refs: Vec<StepPieceRef>,
    pub busy: bool,
    pub running: bool,
    pub status: String,
    pub progress: usize,
    pub elapsed: f64,
    pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}
impl Workspace {
    pub fn update_panels(&mut self, results: &[(f64, NestingResult)]) {
        let mut combined = NestingResult::default();
        let mut materials = vec![];
        for (thickness, result) in results {
            for sheet in result.sheets.iter().filter(|s| !s.placements.is_empty()) {
                materials.push(*thickness);
                combined.sheets.push(sheet.clone());
            }
            combined.unplaced.extend(result.unplaced.iter().copied());
        }
        self.saved.sheet_materials = combined
            .sheets
            .iter()
            .map(|s| crate::cam::panel_material(&self.saved, s))
            .collect();
        self.saved.result = Some(combined);
        self.saved.sheet_thicknesses = materials;
    }
    pub fn persist(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let temp = self.dir.join("workspace.tmp");
        std::fs::write(&temp, serde_json::to_vec(&self.saved)?)?;
        std::fs::rename(temp, self.dir.join("workspace.json"))?;
        Ok(())
    }
    pub fn invalidate(&mut self) {
        self.saved.result = None;
        self.saved.sheet_thicknesses.clear();
        self.saved.sheet_materials.clear();
        self.saved.cam_plan = None;
        self.saved.tabs.clear();
        self.saved.revision += 1;
        self.status = "Ready to nest".into();
        self.progress = 0;
        self.elapsed = 0.;
    }
    pub fn expanded(&self) -> (Vec<Piece>, Vec<StepPieceRef>, Vec<RotationRange>) {
        let mut pieces = vec![];
        let mut refs = vec![];
        let mut rotations = vec![];
        for part in self
            .saved
            .parts
            .iter()
            .filter(|p| p.enabled && self.saved.includes(p))
        {
            for copy in 0..part.piece.quantity {
                let mut piece = part.piece.clone();
                piece.id = PieceId(part.piece.id.0 * 1001 + copy as u64 + 1);
                piece.quantity = 1;
                if let Some(r) = self.refs.iter().find(|r| r.piece_id == part.piece.id) {
                    let mut r = r.clone();
                    r.piece_id = piece.id;
                    refs.push(r);
                }
                rotations.push(match part.rotation.as_str() {
                    "fixed" => RotationRange::None,
                    "quarter" => RotationRange::Discrete(vec![
                        0.,
                        std::f32::consts::FRAC_PI_2,
                        std::f32::consts::PI,
                        3. * std::f32::consts::FRAC_PI_2,
                    ]),
                    _ => RotationRange::Continuous,
                });
                pieces.push(piece);
            }
        }
        (pieces, refs, rotations)
    }
    pub fn restore_parts(&mut self, pieces: Vec<Piece>, refs: Vec<StepPieceRef>) {
        let migrate = self.saved.thickness.is_some()
            || self.saved.parts.len() != pieces.len()
            || self.saved.parts.iter().any(|p| p.thickness_mm.is_none())
            || self
                .saved
                .result
                .as_ref()
                .is_some_and(|r| r.sheets.len() != self.saved.sheet_thicknesses.len());
        let previous: HashMap<_, _> = self.saved.parts.iter().map(|p| (p.piece.id, p)).collect();
        self.saved.parts = pieces
            .into_iter()
            .map(|mut piece| {
                let old = previous.get(&piece.id);
                if let Some(old) = old {
                    piece.quantity = old.piece.quantity;
                }
                let thickness_mm = refs
                    .iter()
                    .find(|r| r.piece_id == piece.id)
                    .map(|r| r.thickness_mm);
                Part {
                    piece,
                    enabled: old.map_or(true, |p| p.enabled),
                    rotation: old.map_or_else(|| "continuous".into(), |p| p.rotation.clone()),
                    thickness_mm,
                }
            })
            .collect();
        self.refs = refs;
        self.saved.thickness = None;
        if migrate {
            self.invalidate();
        }
    }
    pub fn panel_jobs(&self) -> Vec<(f64, Vec<Piece>, Vec<RotationRange>)> {
        let (pieces, refs, rotations) = self.expanded();
        let mut groups =
            std::collections::BTreeMap::<(String, i64), (Vec<Piece>, Vec<RotationRange>)>::new();
        for (piece, rotation) in pieces.into_iter().zip(rotations) {
            if let Some(r) = refs.iter().find(|r| r.piece_id == piece.id) {
                let group = groups
                    .entry((
                        crate::cam::material_for(&self.saved, (piece.id.0 - 1) / 1001),
                        thickness_key(r.thickness_mm),
                    ))
                    .or_default();
                group.0.push(piece);
                group.1.push(rotation);
            }
        }
        groups
            .into_iter()
            .map(|(key, (pieces, rotations))| {
                (key.1 as f64 * THICKNESS_BUCKET_MM, pieces, rotations)
            })
            .collect()
    }
    pub fn tab_map(&self) -> HashMap<(usize, usize), Vec<Tab>> {
        self.saved
            .tabs
            .iter()
            .map(|t| ((t.sheet, t.placement), t.tabs.clone()))
            .collect()
    }
    pub fn generate_tabs(&mut self) {
        self.saved.tabs.clear();
        let (pieces, _, _) = self.expanded();
        if let Some(result) = &self.saved.result {
            for (si, sheet) in result.sheets.iter().enumerate() {
                let mut config = self.saved.settings.tabs.clone();
                if let Some(depth) = self.saved.panel_depth(sheet) {
                    config.material_thickness = depth;
                }
                for (pi, p) in sheet.placements.iter().enumerate() {
                    if let Some(piece) = pieces.iter().find(|piece| piece.id == p.piece_id) {
                        let rotated = polygon_ops::rotate(
                            &piece.polygon,
                            p.rotation,
                            Point2D { x: 0., y: 0. },
                        );
                        let placed = polygon_ops::translate(&rotated, p.x, p.y);
                        self.saved.tabs.push(TabSet {
                            sheet: si,
                            placement: pi,
                            tabs: tabs::generate_tabs(&placed, &config),
                        });
                    }
                }
            }
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        // Never send the host's filesystem paths to the browser.
        let mut saved = self.saved.clone();
        saved.source = None;
        for part in &mut saved.parts {
            part.piece.source_file = PathBuf::new();
        }
        let (mut pieces, _, _) = self.expanded();
        for p in &mut pieces {
            p.source_file = PathBuf::new();
        }
        serde_json::json!({"workspace":saved,"pieces":pieces,"max_thickness_mm":MAX_THICKNESS_MM,"thickness_bucket_mm":THICKNESS_BUCKET_MM,"running":self.running,"busy":self.busy,
            "status":self.status,"iteration":self.progress,"elapsed":self.elapsed})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_thickness_different_materials_are_separate_jobs() {
        let (mut w, _, _) = sample();
        for p in &mut w.saved.parts {
            p.enabled = p.piece.id.0 < 2;
            p.thickness_mm = Some(12.);
        }
        for r in &mut w.refs {
            r.thickness_mm = 12.;
        }
        w.saved.settings.cam.material = "plywood".into();
        w.saved
            .settings
            .cam
            .part_materials
            .insert("1".into(), "mdf".into());
        let jobs = w.panel_jobs();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].0, 12.);
        assert_eq!(jobs[1].0, 12.);
        w.saved.settings.cam.excluded_groups.push("mdf:12".into());
        assert_eq!(w.panel_jobs().len(), 1);
    }
    fn sample() -> (Workspace, Vec<Piece>, Vec<StepPieceRef>) {
        let mut parts = vec![];
        let mut refs = vec![];
        for (id, thickness) in [2., 5., 30., 30.01, 40.].into_iter().enumerate() {
            let piece = Piece {
                id: PieceId(id as u64),
                quantity: if id == 0 { 2 } else { 1 },
                source_file: PathBuf::new(),
                polygon: Polygon {
                    outer: vec![
                        Point2D { x: 0., y: 0. },
                        Point2D { x: 10., y: 0. },
                        Point2D { x: 10., y: 10. },
                        Point2D { x: 0., y: 10. },
                    ],
                    holes: vec![],
                },
            };
            refs.push(StepPieceRef {
                piece_id: piece.id,
                solid_key: thetta_step::StepSolidKey(id as u32),
                source_path: PathBuf::new(),
                thickness_mm: thickness,
            });
            parts.push(Part {
                piece,
                enabled: true,
                rotation: "fixed".into(),
                thickness_mm: Some(thickness),
            });
        }
        let pieces = parts.iter().map(|p| p.piece.clone()).collect();
        (
            Workspace {
                dir: PathBuf::new(),
                saved: Saved {
                    parts,
                    ..Default::default()
                },
                model: None,
                refs: refs.clone(),
                busy: false,
                running: false,
                status: String::new(),
                progress: 0,
                elapsed: 0.,
                cancel: None,
            },
            pieces,
            refs,
        )
    }
    #[test]
    fn jobs_keep_materials_separate_and_include_30mm_but_exclude_thicker_parts() {
        let (mut workspace, _, refs) = sample();
        workspace.refs = refs;
        let jobs = workspace.panel_jobs();
        assert_eq!(
            jobs.iter().map(|(t, _, _)| *t).collect::<Vec<_>>(),
            vec![2., 5., 30.]
        );
        assert_eq!(jobs.iter().map(|(_, p, _)| p.len()).sum::<usize>(), 4);
        assert_eq!(
            jobs[0].1.iter().map(|p| p.id.0).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(jobs[1].1[0].id, PieceId(1002));
        workspace.saved.parts[0].enabled = false;
        assert_eq!(workspace.expanded().0.len(), 2);
    }
    #[test]
    fn restoring_a_legacy_filter_recovers_all_parts_and_preserves_edits() {
        let (mut workspace, pieces, refs) = sample();
        workspace.saved.parts.truncate(1);
        workspace.saved.parts[0].piece.quantity = 3;
        workspace.saved.parts[0].enabled = false;
        workspace.saved.parts[0].thickness_mm = None;
        workspace.saved.thickness = Some(2.);
        workspace.saved.result = Some(NestingResult::default());
        workspace.restore_parts(pieces.clone(), refs.clone());
        assert_eq!(workspace.saved.parts.len(), 5);
        assert!(!workspace.saved.parts[0].enabled);
        assert_eq!(workspace.saved.parts[0].piece.quantity, 3);
        assert_eq!(workspace.saved.parts[0].rotation, "fixed");
        assert!(workspace.saved.thickness.is_none());
        assert!(workspace.saved.result.is_none());
        assert!(
            workspace
                .saved
                .parts
                .iter()
                .all(|p| p.thickness_mm.is_some())
        );
        workspace.saved.result = Some(NestingResult {
            sheets: vec![Sheet::default()],
            unplaced: vec![],
        });
        workspace.saved.sheet_thicknesses = vec![5.];
        workspace.restore_parts(pieces, refs);
        assert!(
            workspace.saved.result.is_some(),
            "restoring again must preserve the grouped layout"
        );
    }
}
