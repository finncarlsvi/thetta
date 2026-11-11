//! Versioned cutting evidence, private machine inventory, and reproducible plans.
use crate::model::Saved;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use thetta_geometry::types::{Tool, ToolTable};

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CamSettings {
    pub automatic: bool,
    pub machine_id: String,
    pub material: String,
    pub part_materials: BTreeMap<String, String>,
    pub excluded_groups: Vec<String>,
    pub overrides: BTreeMap<String, CutOverride>,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CutOverride {
    pub tool_number: Option<u32>,
    pub rpm: Option<u32>,
    pub feed: Option<f64>,
    pub ramp_feed: Option<f64>,
    pub step_down: Option<f64>,
    pub step_over: Option<f64>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub confirmed: bool,
    pub rpm_min: u32,
    pub rpm_max: u32,
    pub feed_max: f64,
    pub plunge_max: f64,
    pub step_down_max: f64,
    pub workholding: String,
    pub chip_removal: String,
    pub notes: String,
}
impl Default for Machine {
    fn default() -> Self {
        Self{id:"meitill".into(),name:"Meitill · LinuxCNC".into(),revision:1,confirmed:false,rpm_min:0,rpm_max:24000,feed_max:0.,plunge_max:0.,step_down_max:0.,workholding:String::new(),chip_removal:String::new(),notes:"24000 RPM command ceiling imported from meitill.ini; practical limits require confirmation. No live machine control.".into()}
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Installed {
    pub definition: Tool,
    pub model: Option<String>,
    #[serde(default)]
    pub substrate: Option<String>,
    pub kind: String,
    pub cutting_length: Option<f64>,
    pub compression_min: Option<f64>,
    pub rpm_max: Option<u32>,
    pub geometry_confirmed: bool,
    pub available: bool,
    pub revision: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub revision: u64,
    pub model: Option<String>,
    #[serde(default)]
    pub tool_number: Option<u32>,
    #[serde(default)]
    pub machine_id: Option<String>,
    pub family: String,
    pub material: String,
    pub operation: String,
    pub source: String,
    pub confidence: String,
    pub diameter: f64,
    pub reference_rpm: u32,
    pub chip_min: f64,
    pub chip_max: f64,
    pub feed_cap: f64,
    pub ramp_ratio: f64,
    pub step_down_ratio: f64,
    pub step_over: f64,
    pub ramp_angle: f64,
    pub feed_per_rev: Option<f64>,
    pub notes: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Cut {
    pub operation: String,
    pub tool: Tool,
    pub recipe: Recipe,
    pub step_down: f64,
    pub depths: Vec<f64>,
    pub step_over: f64,
    pub ramp_angle: f64,
    pub entry: String,
    pub evidence: String,
    pub estimated: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct PanelPlan {
    pub index: usize,
    pub material: String,
    pub thickness: f64,
    pub cuts: Vec<Cut>,
    pub issues: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Plan {
    pub fingerprint: String,
    pub ready: bool,
    pub accepted: bool,
    pub requires_review: bool,
    pub machine: Machine,
    pub panels: Vec<PanelPlan>,
    pub issues: Vec<String>,
    pub catalog_revision: String,
}
#[derive(Serialize)]
pub struct CatalogView {
    pub revision: String,
    pub materials: serde_json::Value,
    pub models: serde_json::Value,
    pub sources: serde_json::Value,
    pub recipes: Vec<Recipe>,
    pub machine: Machine,
    pub tools: Vec<Installed>,
}
const CATALOG_REVISION: &str = "2026-11-10.1";
pub struct Catalog {
    db: Connection,
}
fn encoded<T: Serialize>(x: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(x)?)
}
impl Catalog {
    pub fn open(root: &Path) -> anyhow::Result<Self> {
        let db = Connection::open(root.join("cam.sqlite"))?;
        let version: u32 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
        anyhow::ensure!(
            version <= 1,
            "CAM database was created by a newer application; restore a compatible image"
        );
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
   CREATE TABLE IF NOT EXISTS catalog_records(kind TEXT NOT NULL,id TEXT NOT NULL,revision TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(kind,id,revision));
   CREATE TABLE IF NOT EXISTS machines(owner TEXT NOT NULL,id TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,id));
   CREATE TABLE IF NOT EXISTS installed_tools(owner TEXT NOT NULL,machine TEXT NOT NULL,number INTEGER NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,machine,number));
   CREATE TABLE IF NOT EXISTS shop_recipes(owner TEXT NOT NULL,id TEXT NOT NULL,revision INTEGER NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,id,revision));
   CREATE TABLE IF NOT EXISTS cutting_plans(owner TEXT NOT NULL,workspace TEXT NOT NULL,fingerprint TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(owner,workspace,fingerprint)); PRAGMA user_version=1;")?;
        let seed: serde_json::Value =
            serde_json::from_str(include_str!("../data/cam-catalog-v1.json"))?;
        let seeded: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM catalog_records WHERE revision=?1)",
            [CATALOG_REVISION],
            |r| r.get(0),
        )?;
        if !seeded {
            db.execute_batch("BEGIN IMMEDIATE")?;
            for kind in ["sources", "materials", "models", "recipes"] {
                for row in seed[kind].as_array().unwrap() {
                    db.execute(
                        "INSERT OR IGNORE INTO catalog_records VALUES(?1,?2,?3,?4)",
                        params![
                            kind,
                            row["id"].as_str().unwrap(),
                            seed["revision"].as_str().unwrap(),
                            row.to_string()
                        ],
                    )?;
                }
            }
            db.execute_batch("COMMIT")?;
        }
        Ok(Self { db })
    }
    fn records(&self, kind: &str) -> anyhow::Result<serde_json::Value> {
        let mut stmt = self.db.prepare(
            "SELECT payload FROM catalog_records WHERE kind=?1 AND revision=?2 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![kind, CATALOG_REVISION], |r| r.get::<_, String>(0))?;
        let mut out = vec![];
        for r in rows {
            out.push(serde_json::from_str::<serde_json::Value>(&r?)?);
        }
        Ok(out.into())
    }
    pub fn machine(&self, owner: &str, id: &str) -> anyhow::Result<Machine> {
        let id = if id.is_empty() { "meitill" } else { id };
        let payload = self.db.query_row(
            "SELECT payload FROM machines WHERE owner=?1 AND id=?2",
            params![owner, id],
            |r| r.get::<_, String>(0),
        );
        match payload {
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let mut m = Machine::default();
                m.id = id.into();
                Ok(m)
            }
            Err(e) => Err(e.into()),
        }
    }
    pub fn save_machine(&self, owner: &str, mut m: Machine) -> anyhow::Result<Machine> {
        anyhow::ensure!(
            !m.id.is_empty() && m.id.len() < 80,
            "Machine ID is required"
        );
        anyhow::ensure!(
            m.rpm_min <= m.rpm_max && m.rpm_max > 0 && m.rpm_max <= 24000,
            "Spindle range must be within the imported 24000 RPM ceiling"
        );
        anyhow::ensure!(
            m.feed_max <= 25399.98 && m.plunge_max <= 25399.98,
            "Practical feeds exceed LinuxCNC's 25399.98 mm/min trajectory ceiling"
        );
        for x in [m.feed_max, m.plunge_max, m.step_down_max] {
            anyhow::ensure!(
                x.is_finite() && x >= 0.,
                "Machine limits must be nonnegative"
            );
        }
        if m.confirmed {
            anyhow::ensure!(
                m.rpm_min > 0
                    && m.feed_max > 0.
                    && m.plunge_max > 0.
                    && m.step_down_max > 0.
                    && !m.workholding.trim().is_empty()
                    && !m.chip_removal.trim().is_empty(),
                "Confirm spindle range, practical feeds, stepdown, workholding and chip removal"
            );
        }
        m.revision = self.machine(owner, &m.id)?.revision + 1;
        self.db.execute("INSERT INTO machines VALUES(?1,?2,?3) ON CONFLICT(owner,id) DO UPDATE SET payload=excluded.payload",params![owner,m.id,encoded(&m)?])?;
        Ok(m)
    }
    pub fn inventory(
        &self,
        owner: &str,
        machine: &str,
        legacy: &ToolTable,
    ) -> anyhow::Result<Vec<Installed>> {
        let machine = if machine.is_empty() {
            "meitill"
        } else {
            machine
        };
        for t in &legacy.tools {
            let model = if t.name.contains("46216-K") {
                Some("amana-46216-K")
            } else if t.name.contains("46010-K") {
                Some("amana-46010-K")
            } else if t.name.contains("46294-K") {
                Some("amana-46294-K")
            } else {
                None
            };
            let kind = match model {
                Some("amana-46216-K") => "flat",
                Some("amana-46010-K") => "compression",
                Some("amana-46294-K") => "ball",
                _ if t.name.to_lowercase().contains("drill") => "drill",
                _ if t.name.to_lowercase().contains("probe") => "probe",
                _ if t.name.to_lowercase().contains("compression") => "compression",
                _ if t.name.to_lowercase().contains("planer") => "planer",
                _ if t.name.to_lowercase().contains("degree") => "v_bit",
                _ => "unknown",
            };
            let tool = Installed {
                definition: t.clone(),
                model: model.map(str::to_string),
                substrate: model.map(|_| "carbide".into()),
                kind: kind.into(),
                cutting_length: None,
                compression_min: None,
                rpm_max: None,
                geometry_confirmed: false,
                available: true,
                revision: 1,
            };
            self.db.execute(
                "INSERT OR IGNORE INTO installed_tools VALUES(?1,?2,?3,?4)",
                params![owner, machine, t.number, encoded(&tool)?],
            )?;
        }
        let mut stmt = self.db.prepare(
            "SELECT payload FROM installed_tools WHERE owner=?1 AND machine=?2 ORDER BY number",
        )?;
        let rows = stmt.query_map(params![owner, machine], |r| r.get::<_, String>(0))?;
        let mut out = vec![];
        for row in rows {
            out.push(serde_json::from_str(&row?)?);
        }
        Ok(out)
    }
    pub fn save_tool(
        &self,
        owner: &str,
        machine: &str,
        mut t: Installed,
    ) -> anyhow::Result<Installed> {
        anyhow::ensure!(
            t.definition.number > 0,
            "Installed cutters need a positive tool number"
        );
        t.definition
            .validate_definition()
            .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            [
                "unknown",
                "flat",
                "compression",
                "o_flute",
                "ball",
                "v_bit",
                "planer",
                "probe",
                "drill"
            ]
            .contains(&t.kind.as_str()),
            "Unknown cutter type"
        );
        if let Some(id) = &t.model {
            let models = self.records("models")?;
            let model = models
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"].as_str() == Some(id))
                .ok_or_else(|| anyhow::anyhow!("Unknown tool model"))?;
            anyhow::ensure!(
                model["kind"].as_str() == Some(t.kind.as_str())
                    && (model["diameter"].as_f64().unwrap() - t.definition.diameter).abs() < 0.01
                    && model["flutes"].as_u64() == Some(t.definition.flute_count as u64),
                "Installed geometry must match the selected model"
            );
        }
        for x in [t.cutting_length, t.compression_min] {
            if let Some(x) = x {
                anyhow::ensure!(x.is_finite() && x > 0., "Tool geometry must be positive");
            }
        }
        if t.geometry_confirmed {
            anyhow::ensure!(
                t.definition.diameter > 0.
                    && t.definition.flute_count > 0
                    && t.cutting_length.is_some()
                    && t.rpm_max.is_some_and(|r| r > 0)
                    && t.substrate
                        .as_deref()
                        .is_some_and(|s| ["carbide", "hss", "other"].contains(&s)),
                "Confirm cutter material, diameter, flutes, cutting reach, and maximum RPM"
            );
            if t.kind == "compression" {
                anyhow::ensure!(
                    t.compression_min.is_some(),
                    "Compression transition length is required"
                );
            }
        }
        let existing = self.inventory(owner, machine, &ToolTable::default())?;
        t.revision = existing
            .iter()
            .find(|x| x.definition.number == t.definition.number)
            .map_or(1, |x| x.revision + 1);
        self.db.execute("INSERT INTO installed_tools VALUES(?1,?2,?3,?4) ON CONFLICT(owner,machine,number) DO UPDATE SET payload=excluded.payload",params![owner,machine,t.definition.number,encoded(&t)?])?;
        Ok(t)
    }
    pub fn view(&self, owner: &str, saved: &Saved) -> anyhow::Result<CatalogView> {
        let mut recipes: Vec<Recipe> = serde_json::from_value(self.records("recipes")?)?;
        let mut stmt=self.db.prepare("SELECT payload FROM shop_recipes WHERE owner=?1 AND revision=(SELECT MAX(r2.revision) FROM shop_recipes r2 WHERE r2.owner=shop_recipes.owner AND r2.id=shop_recipes.id) ORDER BY id")?;
        for r in stmt.query_map([owner], |r| r.get::<_, String>(0))? {
            recipes.push(serde_json::from_str(&r?)?);
        }
        Ok(CatalogView {
            revision: CATALOG_REVISION.into(),
            materials: self.records("materials")?,
            models: self.records("models")?,
            sources: self.records("sources")?,
            recipes,
            machine: self.machine(owner, &saved.settings.cam.machine_id)?,
            tools: self.inventory(owner, &saved.settings.cam.machine_id, &saved.settings.tools)?,
        })
    }
    pub fn save_recipe(&self, owner: &str, mut r: Recipe) -> anyhow::Result<Recipe> {
        anyhow::ensure!(
            !r.id.is_empty() && !r.notes.trim().is_empty() && !r.source.trim().is_empty(),
            "Recipe ID, evidence/source and shop notes are required"
        );
        anyhow::ensure!(
            valid_material(&r.material)
                && ["profile", "pocket", "drill"].contains(&r.operation.as_str()),
            "Unsupported material or operation"
        );
        for v in [
            r.diameter,
            r.chip_min,
            r.chip_max,
            r.feed_cap,
            r.ramp_ratio,
            r.step_down_ratio,
            r.step_over,
            r.ramp_angle,
        ] {
            anyhow::ensure!(
                v.is_finite() && v > 0.,
                "Recipe values must be finite and positive"
            );
        }
        anyhow::ensure!(
            r.chip_min <= r.chip_max
                && r.reference_rpm > 0
                && r.step_over <= 0.5
                && r.ramp_ratio <= 1.
                && r.ramp_angle <= 5.
                && r.step_down_ratio <= 1.,
            "Invalid recipe ranges"
        );
        if r.operation == "drill" {
            anyhow::ensure!(
                r.feed_per_rev.is_some_and(|v| v.is_finite() && v > 0.),
                "Drilling requires a feed per revolution"
            );
        }
        let machine = r
            .machine_id
            .as_deref()
            .filter(|m| !m.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Shop recipe needs a machine"))?;
        let number = r
            .tool_number
            .ok_or_else(|| anyhow::anyhow!("Shop recipe needs an installed tool"))?;
        let tools = self.inventory(owner, machine, &ToolTable::default())?;
        anyhow::ensure!(
            tools.iter().any(|t| t.definition.number == number
                && t.kind == r.family
                && (t.definition.diameter - r.diameter).abs() < 0.01),
            "Shop recipe must match an installed cutter"
        );
        r.confidence = "shop".into();
        r.revision = self.db.query_row(
            "SELECT COALESCE(MAX(revision),0)+1 FROM shop_recipes WHERE owner=?1 AND id=?2",
            params![owner, r.id],
            |row| row.get(0),
        )?;
        self.db.execute(
            "INSERT INTO shop_recipes VALUES(?1,?2,?3,?4)",
            params![owner, r.id, r.revision, encoded(&r)?],
        )?;
        Ok(r)
    }
    pub fn store_plan(&self, owner: &str, workspace: &str, p: &Plan) -> anyhow::Result<()> {
        self.db.execute("INSERT INTO cutting_plans VALUES(?1,?2,?3,?4) ON CONFLICT(owner,workspace,fingerprint) DO UPDATE SET payload=excluded.payload",params![owner,workspace,p.fingerprint,encoded(p)?])?;
        Ok(())
    }
}
pub fn valid_material(id: &str) -> bool {
    ["plywood", "mdf", "softwood", "hardwood", "acrylic", "hdpe"].contains(&id)
}
pub fn material_for(saved: &Saved, id: u64) -> String {
    saved
        .settings
        .cam
        .part_materials
        .get(&id.to_string())
        .cloned()
        .unwrap_or_else(|| saved.settings.cam.material.clone())
}
pub fn panel_material(saved: &Saved, sheet: &thetta_geometry::types::Sheet) -> String {
    sheet
        .placements
        .first()
        .map(|p| material_for(saved, (p.piece_id.0 - 1) / 1001))
        .unwrap_or_default()
}
pub fn fingerprint(saved: &Saved, view: &CatalogView) -> anyhow::Result<String> {
    let mut input = saved.clone();
    input.cam_plan = None;
    let bytes = serde_json::to_vec(&(input, view))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
pub fn resolve(saved: &Saved, view: &CatalogView) -> anyhow::Result<Plan> {
    let mut plan = Plan {
        fingerprint: fingerprint(saved, view)?,
        ready: false,
        accepted: false,
        requires_review: false,
        machine: view.machine.clone(),
        panels: vec![],
        issues: vec![],
        catalog_revision: view.revision.clone(),
    };
    if !view.machine.confirmed {
        plan.issues
            .push("Confirm the machine's practical limits and workholding first".into());
    }
    let Some(result) = &saved.result else {
        plan.issues
            .push("Panelize the selected materials first".into());
        return Ok(plan);
    };
    if !result.unplaced.is_empty() {
        plan.issues.push("Some selected parts are unplaced".into());
    }
    for (index, sheet) in result.sheets.iter().enumerate() {
        let mat = panel_material(saved, sheet);
        let depth = saved.panel_depth(sheet).unwrap_or(0.);
        let mut panel = PanelPlan {
            index,
            material: mat.clone(),
            thickness: depth,
            cuts: vec![],
            issues: vec![],
        };
        if !valid_material(&mat) {
            panel
                .issues
                .push("Choose a supported material for every part".into());
        }
        if sheet
            .placements
            .iter()
            .any(|p| material_for(saved, (p.piece_id.0 - 1) / 1001) != mat)
        {
            panel
                .issues
                .push("Materials share a panel: panelize again".into());
        }
        let holes: Vec<_> = sheet
            .placements
            .iter()
            .filter_map(|p| {
                saved
                    .parts
                    .iter()
                    .find(|x| x.piece.id.0 == (p.piece_id.0 - 1) / 1001)
            })
            .flat_map(|p| p.piece.polygon.holes.iter())
            .collect();
        let override_for = |op: &str| saved.settings.cam.overrides.get(&format!("{mat}:{op}"));
        let choose = |op: &str, preferred: Option<u32>, drill_d: Option<f64>| -> Option<Cut> {
            let mut candidates = vec![];
            for t in &view.tools {
                if !t.available
                    || !t.geometry_confirmed
                    || t.cutting_length.unwrap_or(0.) + 1e-6 < depth
                {
                    continue;
                }
                if op == "drill" && t.kind != "drill"
                    || op != "drill"
                        && !["flat", "compression", "o_flute"].contains(&t.kind.as_str())
                {
                    continue;
                }
                if op == "profile"
                    && t.definition.diameter > saved.settings.sheet.separation + 1e-6
                    && sheet.placements.len() > 1
                {
                    continue;
                }
                if op == "drill"
                    && !holes.iter().any(|h| {
                        (thetta_cam::hole_classify::contour_effective_diameter(
                            h,
                            thetta_cam::hole_classify::contour_center(h),
                        ) - t.definition.diameter)
                            .abs()
                            <= 0.02
                            && thetta_cam::hole_classify::circularity(h)
                                > saved.settings.drill.circularity_threshold
                    })
                {
                    continue;
                }
                let ov = override_for(op);
                if ov
                    .and_then(|o| o.tool_number)
                    .is_some_and(|n| n != t.definition.number)
                {
                    continue;
                }
                for r in &view.recipes {
                    if r.operation != op
                        || r.material != mat
                        || (r.diameter - t.definition.diameter).abs() > 0.01
                    {
                        continue;
                    }
                    if r.confidence == "shop"
                        && (r.tool_number != Some(t.definition.number)
                            || r.machine_id.as_deref() != Some(view.machine.id.as_str()))
                    {
                        continue;
                    }
                    if r.confidence != "shop" && t.substrate.as_deref() != Some("carbide") {
                        continue;
                    }
                    let exact = r.model.is_some() && r.model == t.model;
                    if r.confidence != "shop"
                        && r.model.is_none()
                        && (r.family == "flat" && t.definition.flute_count != 2
                            || r.family == "o_flute" && t.definition.flute_count != 1)
                    {
                        continue;
                    }
                    if r.family != t.kind || !exact && r.model.is_some() {
                        continue;
                    }
                    if let Some(c) = calculate(t, r, &view.machine, depth, ov) {
                        let gc = thetta_geometry::types::GcodeConfig {
                            cut_depth: depth,
                            ..saved.settings.gcode.clone()
                        };
                        if op == "pocket" {
                            let mut pc = saved.settings.pocket.clone();
                            pc.step_over = c.step_over;
                            if holes.iter().filter(|h|!drill_d.is_some_and(|d|thetta_cam::hole_classify::circularity(h)>saved.settings.drill.circularity_threshold&&(thetta_cam::hole_classify::contour_effective_diameter(h,thetta_cam::hole_classify::contour_center(h))-d).abs()<=0.02)).any(|h|thetta_cam::passes::pocket(&thetta_geometry::types::PocketRegion{contour:(**h).clone()},&c.tool,&gc,&pc,c.step_down,c.ramp_angle).is_err()){continue;}
                        }
                        if op == "profile" && proof_profile(saved, sheet, &c, depth, index).is_err()
                        {
                            continue;
                        }
                        let rank = (
                            if r.confidence == "shop" {
                                0
                            } else if exact {
                                1
                            } else {
                                2
                            },
                            if Some(t.definition.number) == preferred {
                                0
                            } else {
                                1
                            },
                            t.definition.number,
                            r.id.clone(),
                        );
                        candidates.push((rank, c));
                    }
                }
            }
            candidates.sort_by(|a, b| a.0.cmp(&b.0));
            candidates.into_iter().next().map(|(_, c)| c)
        };
        let profile = choose("profile", None, None);
        if let Some(c) = profile.clone() {
            panel.cuts.push(c);
        } else {
            panel.issues.push("No compatible profile cutter/recipe. Confirm tool reach, flutes, RPM limit, geometry, material, and cutter/entry clearance. Compression cutters need room for a waste helix.".into());
        }
        let drill = if saved.settings.drill.enabled && !holes.is_empty() {
            choose("drill", None, None)
        } else {
            None
        };
        let drill_d = drill.as_ref().map(|c| c.tool.diameter);
        let pending_holes: Vec<_> = holes
            .iter()
            .filter(|h| {
                !drill_d.is_some_and(|d| {
                    thetta_cam::hole_classify::circularity(h)
                        > saved.settings.drill.circularity_threshold
                        && (thetta_cam::hole_classify::contour_effective_diameter(
                            h,
                            thetta_cam::hole_classify::contour_center(h),
                        ) - d)
                            .abs()
                            <= 0.02
                })
            })
            .collect();
        if let Some(c) = drill {
            panel.cuts.push(c);
        }
        if saved.settings.pocket.enabled && !pending_holes.is_empty() {
            if let Some(c) = choose("pocket", profile.as_ref().map(|c| c.tool.number), drill_d) {
                // The planned pocket path must exist for every hole; never silently omit a feature.
                let mut pc = saved.settings.pocket.clone();
                pc.step_over = c.step_over;
                let gc = thetta_geometry::types::GcodeConfig {
                    cut_depth: depth,
                    ..saved.settings.gcode.clone()
                };
                if pending_holes.iter().any(|h| {
                    thetta_cam::pocket::plan_pocket_toolpath(
                        &thetta_geometry::types::PocketRegion {
                            contour: (***h).to_vec(),
                        },
                        &c.tool,
                        &pc,
                        &gc,
                    )
                    .moves
                    .is_empty()
                }) {
                    panel.issues.push("Selected pocket cutter cannot reach every hole; use a smaller cutter or matching drill".into());
                }
                panel.cuts.push(c);
            } else {
                panel
                    .issues
                    .push("No compatible pocket cutter/recipe for the remaining holes".into());
            }
        } else if saved.settings.drill.enabled && !pending_holes.is_empty() {
            panel.issues.push("Some holes have no matching drill. Enable pocketing with a compatible cutter or add a matching drill recipe.".into());
        }
        for c in &panel.cuts {
            plan.requires_review |= c.estimated;
        }
        plan.panels.push(panel);
    }
    plan.ready = plan.issues.is_empty()
        && !plan.panels.is_empty()
        && plan.panels.iter().all(|p| p.issues.is_empty());
    Ok(plan)
}
fn calculate(
    t: &Installed,
    r: &Recipe,
    m: &Machine,
    depth: f64,
    ov: Option<&CutOverride>,
) -> Option<Cut> {
    if !m.confirmed || depth <= 0. || !depth.is_finite() {
        return None;
    }
    let mut rpm = r.reference_rpm.min(m.rpm_max).min(t.rpm_max?);
    let f = t.definition.flute_count as f64;
    if f <= 0. || t.definition.diameter <= 0. || t.definition.number == 0 {
        return None;
    }
    let chip = r.chip_min;
    let per_rev = if r.operation == "drill" {
        r.feed_per_rev?
    } else {
        f * chip
    };
    let feed_limit = if r.operation == "drill" {
        m.feed_max.min(m.plunge_max)
    } else {
        m.feed_max
    };
    rpm = rpm.min((feed_limit.min(r.feed_cap) / per_rev).floor() as u32);
    if rpm < m.rpm_min || rpm == 0 {
        return None;
    }
    let mut tool = t.definition.clone();
    tool.spindle_speed = rpm;
    tool.feed_rate = (rpm as f64 * per_rev).min(r.feed_cap);
    tool.plunge_rate = if r.operation == "drill" {
        tool.feed_rate
    } else {
        (tool.feed_rate * r.ramp_ratio).min(m.plunge_max)
    };
    let mut step = (tool.diameter * r.step_down_ratio)
        .min(m.step_down_max)
        .min(depth);
    if t.kind == "compression" {
        let transition = t.compression_min?;
        if transition > depth || transition > m.step_down_max || transition > tool.diameter {
            return None;
        }
        step = step.max(transition);
    }
    let mut step_over = r.step_over;
    if let Some(o) = ov {
        if let Some(v) = o.rpm {
            tool.spindle_speed = v;
        }
        if let Some(v) = o.feed {
            tool.feed_rate = v;
        }
        if let Some(v) = o.ramp_feed {
            tool.plunge_rate = v;
        }
        if let Some(v) = o.step_down {
            step = v;
        }
        if let Some(v) = o.step_over {
            step_over = v;
        }
    }
    if tool.validate().is_err()
        || tool.spindle_speed < m.rpm_min
        || tool.spindle_speed > m.rpm_max.min(t.rpm_max?)
        || tool.feed_rate < 1.
        || tool.plunge_rate < 1.
        || tool.feed_rate > m.feed_max
        || tool.plunge_rate > m.plunge_max
        || !(0.0..=0.5).contains(&step_over)
        || step_over == 0.
        || !step.is_finite()
        || step <= 0.
        || step > m.step_down_max
        || step > tool.diameter
        || !tool.feed_rate.is_finite()
    {
        return None;
    }
    if t.kind == "compression" && step < t.compression_min? {
        return None;
    }
    let chip_actual = if r.operation == "drill" {
        tool.feed_rate / tool.spindle_speed as f64
    } else {
        tool.feed_rate / (tool.spindle_speed as f64 * f)
    };
    if ov.is_none() && chip_actual + 1e-6 < chip * 0.95 {
        return None;
    }
    let depths = thetta_cam::passes::depths(depth, step).ok()?;
    Some(Cut {
        operation: r.operation.clone(),
        tool,
        recipe: r.clone(),
        step_down: step,
        depths,
        step_over,
        ramp_angle: r.ramp_angle,
        entry: if r.operation == "drill" {
            "peck"
        } else if t.kind == "compression" && r.operation == "profile" {
            "waste helix"
        } else {
            "contour ramp"
        }
        .into(),
        evidence: format!("{} rev {} · {}", r.id, r.revision, r.source),
        estimated: r.confidence == "estimate" || r.confidence == "manufacturer" || ov.is_some(),
    })
}

fn proof_profile(
    saved: &Saved,
    sheet: &thetta_geometry::types::Sheet,
    cut: &Cut,
    depth: f64,
    sheet_index: usize,
) -> Result<(), String> {
    use thetta_geometry::{polygon_ops, types::*};
    let solids: Vec<_> = sheet
        .placements
        .iter()
        .filter_map(|p| {
            saved
                .parts
                .iter()
                .find(|x| x.piece.id.0 == (p.piece_id.0 - 1) / 1001)
                .map(|part| {
                    polygon_ops::translate(
                        &polygon_ops::rotate(
                            &part.piece.polygon,
                            p.rotation,
                            Point2D { x: 0., y: 0. },
                        ),
                        p.x,
                        p.y,
                    )
                })
        })
        .collect();
    let gc = GcodeConfig {
        cut_depth: depth,
        ..saved.settings.gcode.clone()
    };
    for (index, poly) in solids.iter().enumerate() {
        let tabs = saved
            .tabs
            .iter()
            .find(|t| t.sheet == sheet_index && t.placement == index)
            .map(|t| t.tabs.as_slice())
            .unwrap_or(&[]);
        let path = thetta_cam::passes::profile_entry(
            poly,
            tabs,
            &cut.tool,
            &gc,
            &saved.settings.profile,
            cut.step_down,
            cut.ramp_angle,
            cut.entry == "waste helix",
        )?;
        thetta_cam::passes::validate_profile_clearance(&path, &solids, cut.tool.diameter)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::{test_shapes, types::*};
    fn machine() -> Machine {
        Machine {
            confirmed: true,
            rpm_min: 6000,
            rpm_max: 24000,
            feed_max: 12000.,
            plunge_max: 3000.,
            step_down_max: 4.,
            workholding: "clamps and tabs".into(),
            chip_removal: "extraction".into(),
            ..Default::default()
        }
    }
    fn installed() -> Installed {
        Installed {
            definition: Tool {
                number: 5,
                name: "1/2 46216-K".into(),
                diameter: 12.7,
                flute_count: 3,
                feed_rate: 0.,
                plunge_rate: 0.,
                spindle_speed: 0,
            },
            model: Some("amana-46216-K".into()),
            substrate: Some("carbide".into()),
            kind: "flat".into(),
            cutting_length: Some(38.1),
            compression_min: None,
            rpm_max: Some(24000),
            geometry_confirmed: true,
            available: true,
            revision: 1,
        }
    }
    fn saved() -> Saved {
        let mut s = Saved::default();
        s.settings.cam.material = "plywood".into();
        s.settings.cam.automatic = true;
        s.settings.sheet.separation = 20.;
        s.parts = vec![crate::model::Part {
            piece: Piece {
                id: PieceId(0),
                polygon: test_shapes::rectangle(100., 60.),
                quantity: 1,
                source_file: Default::default(),
            },
            enabled: true,
            rotation: "continuous".into(),
            thickness_mm: Some(12.),
        }];
        s.result = Some(NestingResult {
            sheets: vec![Sheet {
                placements: vec![Placement {
                    piece_id: PieceId(1),
                    x: 50.,
                    y: 50.,
                    rotation: 0.,
                }],
                ..Default::default()
            }],
            unplaced: vec![],
        });
        s.sheet_thicknesses = vec![12.];
        s
    }
    fn setup(db: &Catalog) -> CatalogView {
        let t = installed();
        db.save_machine("owner", machine()).unwrap();
        db.save_tool("owner", "meitill", t).unwrap();
        db.view("owner", &saved()).unwrap()
    }
    #[test]
    fn persistent_inventory_is_private_and_migration_is_idempotent() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let table = ToolTable {
            tools: vec![installed().definition],
        };
        assert_eq!(db.inventory("a", "meitill", &table).unwrap().len(), 1);
        assert_eq!(db.inventory("a", "meitill", &table).unwrap().len(), 1);
        assert!(
            db.inventory("b", "meitill", &ToolTable::default())
                .unwrap()
                .is_empty()
        );
        drop(db);
        let db = Catalog::open(d.path()).unwrap();
        assert_eq!(
            db.inventory("a", "meitill", &ToolTable::default())
                .unwrap()
                .len(),
            1
        );
        assert!(!db.machine("b", "meitill").unwrap().confirmed);
    }
    #[test]
    fn future_schema_is_not_downgraded() {
        let d = tempfile::tempdir().unwrap();
        let db = Connection::open(d.path().join("cam.sqlite")).unwrap();
        db.execute_batch("PRAGMA user_version=99;").unwrap();
        drop(db);
        assert!(Catalog::open(d.path()).is_err());
    }
    #[test]
    fn exact_recipe_units_machine_caps_and_passes() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let r = view
            .recipes
            .iter()
            .find(|r| r.id == "46216-plywood-profile")
            .unwrap();
        assert!((r.chip_min - 0.14478).abs() < 1e-9);
        assert!((r.feed_cap - 7620.).abs() < 1e-9);
        let mut m = view.machine.clone();
        m.feed_max = 5000.;
        let c = calculate(&installed(), r, &m, 12., None).unwrap();
        assert!(c.tool.feed_rate <= 5000.);
        assert!((c.tool.feed_rate / (c.tool.spindle_speed as f64 * 3.) - r.chip_min).abs() < 1e-8);
        assert_eq!(c.depths, vec![4., 8., 12.]);
        m.rpm_min = 18000;
        assert!(calculate(&installed(), r, &m, 12., None).is_none());
    }
    #[test]
    fn complete_plan_requires_review_and_changes_invalidate_it() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let s = saved();
        let p = resolve(&s, &view).unwrap();
        assert!(
            p.ready,
            "{:?}",
            p.panels.iter().map(|p| &p.issues).collect::<Vec<_>>()
        );
        assert!(p.requires_review);
        assert!(!p.accepted);
        assert_eq!(p.panels[0].cuts[0].tool.number, 5);
        db.store_plan("owner", "workspace", &p).unwrap();
        let mut changed = s.clone();
        changed.settings.tabs.tab_height += 0.5;
        assert_ne!(fingerprint(&changed, &view).unwrap(), p.fingerprint);
        let mut changed_view = db.view("owner", &s).unwrap();
        changed_view.machine.feed_max -= 100.;
        assert_ne!(fingerprint(&s, &changed_view).unwrap(), p.fingerprint);
    }
    #[test]
    fn no_guessing_unknown_geometry_wrong_material_or_ball_end() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let mut view = setup(&db);
        let mut s = saved();
        view.tools[0].geometry_confirmed = false;
        assert!(!resolve(&s, &view).unwrap().ready);
        view.tools[0].geometry_confirmed = true;
        view.tools[0].kind = "ball".into();
        assert!(!resolve(&s, &view).unwrap().ready);
        view.tools[0].kind = "flat".into();
        s.settings.cam.material = "acrylic".into();
        assert!(!resolve(&s, &view).unwrap().ready);
    }
    #[test]
    fn enabled_holes_cannot_silently_disappear_without_a_matching_tool() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let mut s = saved();
        s.settings.drill.enabled = true;
        s.settings.pocket.enabled = false;
        s.parts[0].piece.polygon.holes.push(vec![
            Point2D { x: 20., y: 20. },
            Point2D { x: 25., y: 20. },
            Point2D { x: 25., y: 25. },
            Point2D { x: 20., y: 25. },
        ]);
        let p = resolve(&s, &view).unwrap();
        assert!(!p.ready);
        assert!(
            p.panels[0]
                .issues
                .iter()
                .any(|s| s.contains("no matching drill"))
        );
        // Profile-only jobs remain an explicit option.
        s.settings.drill.enabled = false;
        assert!(resolve(&s, &view).unwrap().ready);
    }
    #[test]
    fn compression_transition_and_overrides_respect_limits() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let r = view
            .recipes
            .iter()
            .find(|r| r.id == "46010-plywood-profile")
            .unwrap();
        let mut t = installed();
        t.kind = "compression".into();
        t.model = Some("amana-46010-K".into());
        t.definition.diameter = 9.525;
        t.compression_min = Some(9.);
        assert!(calculate(&t, r, &machine(), 12., None).is_none());
        let mut m = machine();
        m.step_down_max = 10.;
        let c = calculate(&t, r, &m, 12., None).unwrap();
        assert!(c.depths[0] >= 9.);
        assert_eq!(c.entry, "waste helix");
        let bad = CutOverride {
            rpm: Some(30000),
            ..Default::default()
        };
        assert!(calculate(&t, r, &m, 12., Some(&bad)).is_none());
    }
    #[test]
    fn shop_recipes_are_versioned_and_scoped_to_the_machine_and_tool() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let mut r = view
            .recipes
            .iter()
            .find(|r| r.id == "46216-plywood-profile")
            .unwrap()
            .clone();
        r.id = "trial".into();
        r.source = "shop: trial 2026-11-10".into();
        r.notes = "Plywood, clamps, extraction, acceptable finish".into();
        r.machine_id = Some("meitill".into());
        r.tool_number = Some(5);
        let r = db.save_recipe("owner", r).unwrap();
        assert_eq!(r.revision, 1);
        let r = db.save_recipe("owner", r).unwrap();
        assert_eq!(r.revision, 2);
        assert!(
            db.view("other", &saved())
                .unwrap()
                .recipes
                .iter()
                .all(|r| r.confidence != "shop")
        );
        let p = resolve(&saved(), &db.view("owner", &saved()).unwrap()).unwrap();
        assert_eq!(p.panels[0].cuts[0].recipe.id, "trial");
        assert!(!p.requires_review);
    }
    #[test]
    fn drilling_feed_is_per_revolution_not_per_flute() {
        let d = tempfile::tempdir().unwrap();
        let db = Catalog::open(d.path()).unwrap();
        let view = setup(&db);
        let mut r = view.recipes[0].clone();
        r.operation = "drill".into();
        r.feed_per_rev = Some(0.1);
        r.feed_cap = 2000.;
        r.reference_rpm = 10000;
        let c = calculate(&installed(), &r, &machine(), 12., None).unwrap();
        assert_eq!(c.tool.plunge_rate, 1000.);
    }
}
