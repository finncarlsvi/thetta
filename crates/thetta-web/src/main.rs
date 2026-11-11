mod auth;
mod cam;
mod cam_api;
mod model;
mod onshape;
mod packing;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post, put},
};
use model::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    convert::Infallible,
    io::Write,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use thetta_step::{ExtractionConfig, extract_all_pieces, import_step};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};
use uuid::Uuid;

type Shared = Arc<Mutex<Workspace>>;
#[derive(Clone)]
struct App {
    root: PathBuf,
    workspaces: Arc<Mutex<HashMap<String, Shared>>>,
    occt: Arc<Mutex<()>>,
    shutting_down: Arc<AtomicBool>,
    auth: Arc<auth::Auth>,
}
#[derive(Debug)]
struct Error(StatusCode, String);
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        eprintln!("{e:#}");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The operation failed; see server logs".into(),
        )
    }
}
fn bad(s: impl ToString) -> Error {
    Error(StatusCode::BAD_REQUEST, s.to_string())
}
fn idle(w: &Workspace) -> Result<(), Error> {
    if w.busy || w.running {
        Err(Error(
            StatusCode::CONFLICT,
            "Stop the current job and wait for it to finish first".into(),
        ))
    } else {
        Ok(())
    }
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, Error> + Send + 'static,
) -> Result<T, Error> {
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        eprintln!("Worker failed: {e}");
        Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Processing failed".into(),
        )
    })?
}
fn session(app: &App, headers: &HeaderMap) -> Result<Shared, Error> {
    let p = app.auth.principal(headers)?;
    app.workspaces
        .lock()
        .unwrap()
        .get(&p.workspace)
        .cloned()
        .ok_or_else(|| Error(StatusCode::CONFLICT, "Open the workspace first".into()))
}
fn load_workspace(app: &App, id: &str) -> Result<Shared, Error> {
    let mut map = app.workspaces.lock().unwrap();
    if let Some(w) = map.get(id) {
        return Ok(w.clone());
    }
    let dir = app.root.join(id);
    let saved: Saved = if dir.join("workspace.json").exists() {
        serde_json::from_slice(
            &std::fs::read(dir.join("workspace.json")).map_err(anyhow::Error::from)?,
        )
        .map_err(anyhow::Error::from)?
    } else {
        Saved::default()
    };
    let mut w = Workspace {
        dir,
        saved,
        model: None,
        refs: vec![],
        busy: false,
        running: false,
        status: "Ready".into(),
        progress: 0,
        elapsed: 0.,
        cancel: None,
    };
    if let Some(source) = &w.saved.source {
        let _occt = app.occt.lock().unwrap();
        match import_step(&w.dir.join(source), None, &ExtractionConfig::default()) {
            Ok(imported) => {
                let (pieces, refs) = extract_all_pieces(&imported.model).map_err(bad)?;
                w.restore_parts(pieces, refs);
                w.model = Some(imported.model);
                w.status = "Workspace restored".into();
            }
            Err(e) => {
                eprintln!("Restore failed: {e}");
                w.status = "Source could not be restored. Upload the STEP file again.".into();
                w.saved.result = None;
                w.saved.tabs.clear();
            }
        }
    }
    w.persist().map_err(anyhow::Error::from)?;
    let shared = Arc::new(Mutex::new(w));
    map.insert(id.into(), shared.clone());
    Ok(shared)
}
async fn open(State(app): State<App>, headers: HeaderMap) -> Result<Response, Error> {
    let principal = app.auth.principal(&headers)?;
    let w = blocking(move || {
        let w = load_workspace(&app, &principal.workspace)?;
        let mut ws = w.lock().unwrap();
        if ws.saved.settings.tools.tools.is_empty() {
            let inventory = cam::Catalog::open(&app.root)?.inventory(
                &principal.user_id,
                if ws.saved.settings.cam.machine_id.is_empty() {
                    "meitill"
                } else {
                    &ws.saved.settings.cam.machine_id
                },
                &ws.saved.settings.tools,
            )?;
            if !inventory.is_empty() {
                ws.saved.settings.tools.tools =
                    inventory.into_iter().map(|t| t.definition).collect();
                ws.saved.revision += 1;
                ws.saved.cam_plan = None;
                ws.persist()?;
            }
        }
        drop(ws);
        Ok(w)
    })
    .await?;
    let snap = w.lock().unwrap().snapshot();
    Ok(([("cache-control", "no-store")], Json(snap)).into_response())
}
async fn snapshot(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    let v = w.lock().unwrap().snapshot();
    Ok(Json(v))
}
async fn events(State(app): State<App>, headers: HeaderMap) -> Result<impl IntoResponse, Error> {
    let w = session(&app, &headers)?;
    let stream = async_stream::stream! {
        let mut timer=tokio::time::interval(Duration::from_millis(750));
        let mut previous=String::new();
        loop {
            timer.tick().await;
            if app.shutting_down.load(Ordering::Relaxed) || app.auth.principal(&headers).is_err() { break; }
            let data=w.lock().unwrap().snapshot().to_string();
            if data!=previous { previous=data.clone(); yield Ok::<_,Infallible>(Event::default().data(data)); }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
async fn upload(
    State(app): State<App>,
    headers: HeaderMap,
    mut form: Multipart,
) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    {
        idle(&w.lock().unwrap())?;
    }
    let field = form
        .next_field()
        .await
        .map_err(bad)?
        .ok_or_else(|| bad("Choose a STEP file"))?;
    let filename = field
        .file_name()
        .unwrap_or("source.step")
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("source.step")
        .to_string();
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !["step", "stp"].contains(&ext.as_str()) {
        return Err(bad("Upload a .step or .stp file"));
    }
    let data = field.bytes().await.map_err(bad)?;
    if data.is_empty() {
        return Err(bad("The file is empty"));
    }
    {
        let mut ws = w.lock().unwrap();
        idle(&ws)?;
        ws.busy = true;
        ws.status = "Importing STEP…".into();
    }
    let w2 = w.clone();
    let result = blocking(move || import_bytes(&app, &w2, &filename, &data, None)).await;
    let mut ws = w.lock().unwrap();
    ws.busy = false;
    if let Err(e) = &result {
        ws.status = e.1.clone();
    }
    result?;
    Ok(Json(ws.snapshot()))
}
fn import_bytes(
    app: &App,
    w: &Shared,
    filename: &str,
    data: &[u8],
    onshape: Option<model::OnshapeSource>,
) -> Result<(), Error> {
    let dir = w.lock().unwrap().dir.clone();
    let source = format!("{}.step", Uuid::new_v4());
    let path = dir.join(&source);
    std::fs::write(&path, data).map_err(anyhow::Error::from)?;
    let result = (|| {
        let imported = {
            let _lock = app.occt.lock().unwrap();
            import_step(&path, None, &ExtractionConfig::default())
        }
        .map_err(bad)?;
        let (pieces, refs) = extract_all_pieces(&imported.model).map_err(bad)?;
        let mut ws = w.lock().unwrap();
        let previous = ws.saved.clone();
        let old_model = ws.model.clone();
        let old_refs = ws.refs.clone();
        ws.saved.filename = Some(filename.into());
        ws.saved.source = Some(source);
        ws.saved.thickness = None;
        ws.saved.groups = imported
            .thicknesses
            .iter()
            .map(|t| (t.value_mm, t.count))
            .collect();
        ws.saved.settings.cam.part_materials.clear();
        ws.saved.parts = pieces
            .into_iter()
            .map(|piece| Part {
                thickness_mm: refs
                    .iter()
                    .find(|r| r.piece_id == piece.id)
                    .map(|r| r.thickness_mm),
                piece,
                enabled: true,
                rotation: "continuous".into(),
            })
            .collect();
        ws.model = Some(imported.model);
        ws.refs = refs;
        ws.invalidate();
        ws.saved.onshape = onshape;
        if let Err(e) = ws.persist() {
            ws.saved = previous;
            ws.model = old_model;
            ws.refs = old_refs;
            return Err(anyhow::Error::from(e).into());
        }
        if let Some(old) = previous.source {
            let _ = std::fs::remove_file(dir.join(old));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}
async fn settings(
    State(app): State<App>,
    headers: HeaderMap,
    Json(settings): Json<Settings>,
) -> Result<Json<Value>, Error> {
    settings.validate().map_err(bad)?;
    let w = session(&app, &headers)?;
    blocking(move || {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        let old = &w.saved.settings;
        let layout_changed = serde_json::to_value((
            &old.sheet,
            &old.optimizer,
            &old.mode,
            &old.cam.material,
            &old.cam.part_materials,
            &old.cam.excluded_groups,
        ))
        .unwrap()
            != serde_json::to_value((
                &settings.sheet,
                &settings.optimizer,
                &settings.mode,
                &settings.cam.material,
                &settings.cam.part_materials,
                &settings.cam.excluded_groups,
            ))
            .unwrap();
        let tabs_changed = old.tabs != settings.tabs;
        w.saved.cam_plan = None;
        w.saved.settings = settings;
        if layout_changed {
            w.invalidate();
        } else {
            w.saved.revision += 1;
            if tabs_changed {
                w.saved.tabs.clear();
            }
        }
        w.persist().map_err(anyhow::Error::from)?;
        Ok(Json(w.snapshot()))
    })
    .await
}
#[derive(Deserialize)]
struct PartEdit {
    quantity: u32,
    enabled: bool,
    rotation: String,
}
async fn part(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(edit): Json<PartEdit>,
) -> Result<Json<Value>, Error> {
    if !(1..=1000).contains(&edit.quantity)
        || !["continuous", "quarter", "fixed"].contains(&edit.rotation.as_str())
    {
        return Err(bad("Invalid quantity or rotation"));
    }
    let w = session(&app, &headers)?;
    blocking(move || {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        let target = w
            .saved
            .parts
            .iter()
            .find(|p| p.piece.id.0 == id)
            .ok_or_else(|| bad("Part not found"))?;
        let eligible = w.saved.includes(target);
        let total: u32 = w
            .saved
            .parts
            .iter()
            .filter(|p| p.enabled && w.saved.includes(p) && p.piece.id.0 != id)
            .map(|p| p.piece.quantity)
            .sum();
        if total
            + if edit.enabled && eligible {
                edit.quantity
            } else {
                0
            }
            > 2000
        {
            return Err(bad("At most 2,000 parts per layout"));
        }
        let p = w
            .saved
            .parts
            .iter_mut()
            .find(|p| p.piece.id.0 == id)
            .ok_or_else(|| bad("Part not found"))?;
        p.piece.quantity = edit.quantity;
        p.enabled = edit.enabled;
        p.rotation = edit.rotation;
        w.invalidate();
        w.persist().map_err(anyhow::Error::from)?;
        Ok(Json(w.snapshot()))
    })
    .await
}
#[derive(Deserialize)]
struct Thickness {
    value: Option<f64>,
}
#[derive(Deserialize)]
struct MaterialEdit {
    enabled: bool,
    #[serde(default)]
    material: Option<String>,
}
async fn material(
    State(app): State<App>,
    headers: HeaderMap,
    Path(value): Path<f64>,
    Json(edit): Json<MaterialEdit>,
) -> Result<Json<Value>, Error> {
    if !value.is_finite() || value <= 0. || value > MAX_THICKNESS_MM {
        return Err(bad("Choose a material thickness at or below 30 mm"));
    }
    let w = session(&app, &headers)?;
    blocking(move || {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        let key = thickness_key(value);
        if let Some(material) = &edit.material {
            if !material.is_empty() && !cam::valid_material(material) {
                return Err(bad("Unsupported material"));
            }
            let group = format!("{}:{}", material, key as f64 * THICKNESS_BUCKET_MM);
            if !w.saved.parts.iter().any(|p| {
                p.eligible()
                    && p.thickness_mm.is_some_and(|t| thickness_key(t) == key)
                    && cam::material_for(&w.saved, p.piece.id.0) == *material
            }) {
                return Err(bad("Material group not found"));
            }
            let old = w.saved.settings.cam.excluded_groups.clone();
            let old_thicknesses = w.saved.excluded_thicknesses.clone();
            // Older projects excluded an entire thickness. Convert that selection
            // to material groups before changing one group, preserving the others.
            if old_thicknesses.iter().any(|t| thickness_key(*t) == key) {
                let groups: std::collections::BTreeSet<_> = w
                    .saved
                    .parts
                    .iter()
                    .filter(|p| {
                        p.eligible() && p.thickness_mm.is_some_and(|t| thickness_key(t) == key)
                    })
                    .map(|p| {
                        format!(
                            "{}:{}",
                            cam::material_for(&w.saved, p.piece.id.0),
                            key as f64 * THICKNESS_BUCKET_MM
                        )
                    })
                    .collect();
                w.saved
                    .excluded_thicknesses
                    .retain(|t| thickness_key(*t) != key);
                for group in groups {
                    if !w.saved.settings.cam.excluded_groups.contains(&group) {
                        w.saved.settings.cam.excluded_groups.push(group);
                    }
                }
            }
            w.saved.settings.cam.excluded_groups.retain(|g| g != &group);
            if !edit.enabled {
                w.saved.settings.cam.excluded_groups.push(group);
            }
            let count: u32 = w
                .saved
                .parts
                .iter()
                .filter(|p| p.enabled && w.saved.includes(p))
                .map(|p| p.piece.quantity)
                .sum();
            if count > 2000 {
                w.saved.settings.cam.excluded_groups = old;
                w.saved.excluded_thicknesses = old_thicknesses;
                return Err(bad("At most 2000 parts per layout"));
            }
            w.invalidate();
            w.persist()?;
            return Ok(Json(w.snapshot()));
        }
        if !w
            .saved
            .parts
            .iter()
            .any(|p| p.eligible() && p.thickness_mm.is_some_and(|t| thickness_key(t) == key))
        {
            return Err(bad("Material thickness not found"));
        }
        let mut excluded = w.saved.excluded_thicknesses.clone();
        let was_enabled = !excluded.iter().any(|t| thickness_key(*t) == key);
        if was_enabled == edit.enabled {
            return Ok(Json(w.snapshot()));
        }
        excluded.retain(|t| thickness_key(*t) != key);
        if !edit.enabled {
            excluded.push(key as f64 * THICKNESS_BUCKET_MM);
        }
        excluded.sort_by(f64::total_cmp);
        let count: u32 = w
            .saved
            .parts
            .iter()
            .filter(|p| {
                p.enabled
                    && p.eligible()
                    && p.thickness_mm.is_some_and(|t| {
                        !excluded
                            .iter()
                            .any(|v| thickness_key(*v) == thickness_key(t))
                    })
            })
            .map(|p| p.piece.quantity)
            .sum();
        if count > 2000 {
            return Err(bad("At most 2,000 parts per layout"));
        }
        w.saved.excluded_thicknesses = excluded;
        w.invalidate();
        w.persist().map_err(anyhow::Error::from)?;
        Ok(Json(w.snapshot()))
    })
    .await
}
async fn thickness(
    State(app): State<App>,
    headers: HeaderMap,
    Json(t): Json<Thickness>,
) -> Result<Json<Value>, Error> {
    if t.value.is_some() {
        return Err(bad(
            "All thicknesses up to 30 mm are panelized together in separate arrays",
        ));
    }
    let w = session(&app, &headers)?;
    blocking(move || {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        let model = w.model.as_ref().ok_or_else(|| bad("Upload STEP first"))?;
        let (pieces, refs) = extract_all_pieces(model).map_err(bad)?;
        w.restore_parts(pieces, refs);
        w.invalidate();
        w.persist().map_err(anyhow::Error::from)?;
        Ok(Json(w.snapshot()))
    })
    .await
}
async fn start(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    let (jobs, settings, cancel, revision) = {
        let mut ws = w.lock().unwrap();
        idle(&ws)?;
        let jobs = ws.panel_jobs();
        let count: usize = jobs.iter().map(|(_, pieces, _)| pieces.len()).sum();
        if count == 0 || count > 2000 {
            return Err(bad(
                "Choose between 1 and 2,000 parts at thicknesses up to 30 mm",
            ));
        }
        if ws.model.is_none() {
            return Err(bad("Open a STEP project first"));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        ws.cancel = Some(cancel.clone());
        ws.running = true;
        ws.saved.result = None;
        ws.saved.sheet_thicknesses.clear();
        ws.saved.tabs.clear();
        ws.status = "Panelizing selected thicknesses…".into();
        ws.progress = 0;
        ws.elapsed = 0.;
        (jobs, ws.saved.settings.clone(), cancel, ws.saved.revision)
    };
    let worker = w.clone();
    std::thread::spawn(move || {
        let began = Instant::now();
        let count = jobs.len();
        // Keep one result per material, then flatten without changing sheet-local coordinates.
        let mut results: Vec<_> = jobs
            .iter()
            .map(|(thickness, pieces, _)| {
                (
                    *thickness,
                    thetta_geometry::types::NestingResult {
                        sheets: vec![],
                        unplaced: pieces.iter().map(|p| p.id).collect(),
                    },
                )
            })
            .collect();
        let per_group = (settings.optimizer.time_limit_secs / count as f64).max(0.1);
        // Reserve part of the existing budget for contour-aware completion and
        // coordinated sheet consolidation on EVERY ordinary Panelize run.
        let automatic_budget = if settings.mode == "strip" {
            0.
        } else {
            per_group * 0.25
        };
        let initial_budget = per_group - automatic_budget;
        let mut ok = true;
        let mut last_save = Instant::now();
        for (index, (thickness, pieces, rotations)) in jobs.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let group_cancel = Arc::new(AtomicBool::new(false));
            let group_began = Instant::now();
            let (tx, rx) = std::sync::mpsc::channel();
            let mut config = settings.optimizer.clone();
            config.time_limit_secs = initial_budget;
            let sheet = settings.sheet.clone();
            let fallback_pieces = pieces.clone();
            let fallback_rotations = rotations.clone();
            let fallback_sheet = sheet.clone();
            let mode = if settings.mode == "strip" {
                thetta_nesting::runner::NestingMode::Strip
            } else {
                thetta_nesting::runner::NestingMode::Sheet
            };
            let runner_cancel = group_cancel.clone();
            worker.lock().unwrap().status =
                format!("Panelizing {thickness} mm · {}/{}…", index + 1, count);
            let runner = std::thread::spawn(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    thetta_nesting::optimizer::run_optimizer(
                        pieces,
                        sheet,
                        Some(rotations),
                        config,
                        mode,
                        tx,
                        runner_cancel,
                    );
                }))
                .is_ok()
            });
            loop {
                if group_began.elapsed().as_secs_f64() >= initial_budget
                    || cancel.load(Ordering::Relaxed)
                {
                    group_cancel.store(true, Ordering::Relaxed);
                }
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(update) => {
                        use thetta_nesting::runner::NestingUpdate::*;
                        let mut ws = worker.lock().unwrap();
                        if ws.saved.revision != revision {
                            group_cancel.store(true, Ordering::Relaxed);
                            continue;
                        }
                        match update {
                            Improved(r) | Finished(r) => {
                                results[index].1 = r;
                                ws.update_panels(&results);
                            }
                            Progress { iteration, .. } => ws.progress = iteration,
                        }
                        ws.elapsed = began.elapsed().as_secs_f64();
                        if last_save.elapsed() > Duration::from_secs(2) {
                            if let Err(e) = ws.persist() {
                                eprintln!("Save failed: {e}");
                            }
                            last_save = Instant::now();
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        worker.lock().unwrap().elapsed = began.elapsed().as_secs_f64()
                    }
                    Err(_) => break,
                }
            }
            let group_ok = runner.join().unwrap_or(false);
            if matches!(mode, thetta_nesting::runner::NestingMode::Sheet)
                && !cancel.load(Ordering::Relaxed)
            {
                packing::complete(
                    &mut results[index].1,
                    &fallback_pieces,
                    &fallback_rotations,
                    &fallback_sheet,
                    &cancel,
                );
                if results[index].1.unplaced.is_empty() && !cancel.load(Ordering::Relaxed) {
                    worker.lock().unwrap().status =
                        format!("Consolidating {thickness} mm automatically…");
                    let mut baseline = results[index].1.clone();
                    let baseline_valid = thetta_nesting::improve::validate(
                        &baseline,
                        &fallback_pieces,
                        &fallback_rotations,
                        &fallback_sheet,
                    )
                    .is_ok();
                    // Carry unused time from already consolidated materials forward.
                    let remaining = Duration::from_secs_f64(per_group * (index + 1) as f64)
                        .saturating_sub(began.elapsed());
                    let deadline = Instant::now() + remaining;
                    // Diversify the initial grouping without reusing saved placements.
                    // A quick rectangle seed can expose a merge that the native
                    // baseline's sheet assignments hide. A native layout awaiting
                    // clearance repair is not a valid bound on the sheet count.
                    let mut seed = Default::default();
                    packing::complete(
                        &mut seed,
                        &fallback_pieces,
                        &fallback_rotations,
                        &fallback_sheet,
                        &cancel,
                    );
                    if seed.unplaced.is_empty()
                        && (seed.sheets.len() <= baseline.sheets.len() || !baseline_valid)
                    {
                        let trial = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            thetta_nesting::improve::consolidate(
                                &fallback_pieces,
                                &fallback_rotations,
                                &fallback_sheet,
                                &seed,
                                remaining / 2,
                                &cancel,
                                |_| {},
                                |_| {},
                            )
                        }));
                        if let Ok(Ok(candidate)) = trial {
                            if candidate.sheets.len() < baseline.sheets.len() || !baseline_valid {
                                baseline = candidate;
                            }
                        }
                    }
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    let refined = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        thetta_nesting::improve::consolidate(
                            &fallback_pieces,
                            &fallback_rotations,
                            &fallback_sheet,
                            &baseline,
                            remaining,
                            &cancel,
                            |r| {
                                results[index].1 = r;
                                let mut ws = worker.lock().unwrap();
                                if ws.saved.revision == revision {
                                    ws.update_panels(&results);
                                    ws.elapsed = began.elapsed().as_secs_f64();
                                    if last_save.elapsed() > Duration::from_secs(2) {
                                        if let Err(e) = ws.persist() {
                                            eprintln!("Save failed: {e}");
                                        }
                                        last_save = Instant::now();
                                    }
                                }
                            },
                            |iteration| {
                                let mut ws = worker.lock().unwrap();
                                ws.progress = iteration;
                                ws.elapsed = began.elapsed().as_secs_f64();
                            },
                        )
                    }));
                    match refined {
                        Ok(Ok(r)) => results[index].1 = r,
                        Ok(Err(e)) => {
                            eprintln!("Automatic consolidation failed: {e}");
                            ok = false;
                        }
                        Err(_) => {
                            eprintln!("Automatic consolidation panicked; retaining best layout");
                            ok = false;
                        }
                    }
                }
                let mut ws = worker.lock().unwrap();
                if ws.saved.revision == revision {
                    ws.update_panels(&results);
                }
            }
            ok &= group_ok || results[index].1.unplaced.is_empty();
        }
        let mut ws = worker.lock().unwrap();
        ws.running = false;
        ws.cancel = None;
        ws.elapsed = began.elapsed().as_secs_f64();
        ws.status = if !ok {
            "Optimizer failed. Adjust settings and try again."
        } else if cancel.load(Ordering::Relaxed) {
            "Panelization stopped"
        } else {
            "Selected thickness arrays ready"
        }
        .into();
        if let Err(e) = ws.persist() {
            ws.status = format!("Could not save layout: {e}");
        }
    });
    let snap = w.lock().unwrap().snapshot();
    Ok(Json(snap))
}
async fn improve(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    start_improvement(app, headers, Duration::from_secs(180)).await
}
async fn start_improvement(
    app: App,
    headers: HeaderMap,
    budget: Duration,
) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    let (jobs, mut results, stock, cancel, revision) = {
        let mut ws = w.lock().unwrap();
        idle(&ws)?;
        if ws.saved.settings.mode != "sheet" {
            return Err(bad("Improve layout requires sheet mode"));
        }
        let baseline = ws
            .saved
            .result
            .as_ref()
            .ok_or_else(|| bad("Panelize the selected parts first"))?;
        let jobs = ws.panel_jobs();
        let count: usize = jobs.iter().map(|(_, p, _)| p.len()).sum();
        if count == 0
            || count > 2000
            || !baseline.unplaced.is_empty()
            || baseline.sheets.len() != ws.saved.sheet_thicknesses.len()
        {
            return Err(bad(
                "A complete layout matching the selected parts is required",
            ));
        }
        let mut results = vec![];
        for (thickness, pieces, rotations) in &jobs {
            let result = thetta_geometry::types::NestingResult {
                sheets: baseline
                    .sheets
                    .iter()
                    .zip(&ws.saved.sheet_thicknesses)
                    .filter(|(s, t)| {
                        thickness_key(**t) == thickness_key(*thickness)
                            && s.placements
                                .iter()
                                .all(|p| pieces.iter().any(|x| x.id == p.piece_id))
                    })
                    .map(|(s, _)| s.clone())
                    .collect(),
                unplaced: vec![],
            };
            let zero_gap = thetta_geometry::types::SheetConfig {
                separation: 0.,
                ..ws.saved.settings.sheet.clone()
            };
            thetta_nesting::improve::validate(&result, pieces, rotations, &zero_gap)
                .map_err(bad)?;
            results.push((*thickness, result));
        }
        if results.iter().map(|(_, r)| r.sheets.len()).sum::<usize>() != baseline.sheets.len() {
            return Err(bad("Layout materials do not match the selected parts"));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        ws.cancel = Some(cancel.clone());
        ws.running = true;
        ws.progress = 0;
        ws.elapsed = 0.;
        ws.status = "Improving layout · up to 3 minutes…".into();
        (
            jobs,
            results,
            ws.saved.settings.sheet.clone(),
            cancel,
            ws.saved.revision,
        )
    };
    let worker = w.clone();
    std::thread::spawn(move || {
        let began = Instant::now();
        let groups = jobs.len();
        let mut changed = false;
        let mut failed = false;
        for (index, (thickness, pieces, rotations)) in jobs.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) || began.elapsed() >= budget {
                break;
            }
            let remaining = budget.saturating_sub(began.elapsed());
            let group_budget = remaining.div_f64((groups - index) as f64);
            worker.lock().unwrap().status = format!(
                "Improving {thickness} mm · {}/{} · up to 3 minutes…",
                index + 1,
                groups
            );
            let (tx, rx) = std::sync::mpsc::channel();
            let progress_tx = tx.clone();
            let baseline = results[index].1.clone();
            let stock = stock.clone();
            let cancel_runner = cancel.clone();
            let runner = std::thread::spawn(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    thetta_nesting::improve::improve(
                        &pieces,
                        &rotations,
                        &stock,
                        &baseline,
                        group_budget,
                        &cancel_runner,
                        |r| {
                            let _ = tx.send(thetta_nesting::runner::NestingUpdate::Improved(r));
                        },
                        |iteration| {
                            let _ =
                                progress_tx.send(thetta_nesting::runner::NestingUpdate::Progress {
                                    iteration,
                                    total_overlap: 0.,
                                    best_overlap: 0.,
                                    feasible_count: 0,
                                });
                        },
                    )
                }))
            });
            loop {
                let message = rx.recv_timeout(Duration::from_millis(100));
                let mut ws = worker.lock().unwrap();
                if ws.saved.revision != revision {
                    cancel.store(true, Ordering::Relaxed);
                    break;
                }
                ws.elapsed = began.elapsed().as_secs_f64();
                match message {
                    Ok(thetta_nesting::runner::NestingUpdate::Improved(r)) => {
                        results[index].1 = r;
                        ws.update_panels(&results);
                        ws.saved.tabs.clear();
                        changed = true;
                        if let Err(e) = ws.persist() {
                            eprintln!("Save improved layout failed: {e}");
                            failed = true;
                            cancel.store(true, Ordering::Relaxed);
                        }
                    }
                    Ok(thetta_nesting::runner::NestingUpdate::Progress { iteration, .. }) => {
                        ws.progress = iteration
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    _ => {}
                }
            }
            if !matches!(runner.join(), Ok(Ok(Ok(_)))) {
                failed = true;
                break;
            }
        }
        let mut ws = worker.lock().unwrap();
        ws.running = false;
        ws.cancel = None;
        ws.elapsed = began.elapsed().as_secs_f64();
        ws.status = if failed {
            "Improvement failed; best layout retained"
        } else if cancel.load(Ordering::Relaxed) {
            "Improvement stopped; best layout retained"
        } else if changed {
            "Layout improved"
        } else {
            "No better layout found"
        }
        .into();
        if let Err(e) = ws.persist() {
            ws.status = format!("Could not save layout: {e}");
        }
    });
    let snap = w.lock().unwrap().snapshot();
    Ok(Json(snap))
}

async fn stop(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    let mut w = w.lock().unwrap();
    if let Some(c) = &w.cancel {
        c.store(true, Ordering::Relaxed);
        w.status = "Stopping…".into();
    }
    Ok(Json(w.snapshot()))
}
async fn tabs(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    blocking(move || {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        w.generate_tabs();
        w.persist().map_err(anyhow::Error::from)?;
        Ok(Json(w.snapshot()))
    })
    .await
}
fn zip_files(files: Vec<(String, Vec<u8>)>) -> anyhow::Result<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, data) in files {
        zip.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )?;
        zip.write_all(&data)?;
    }
    Ok(zip.finish()?.into_inner())
}
async fn export(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
) -> Result<Response, Error> {
    if !["step", "gcode"].contains(&kind.as_str()) {
        return Err(bad("Unknown export format"));
    }
    let w = session(&app, &headers)?;
    {
        let mut ws = w.lock().unwrap();
        idle(&ws)?;
        if ws.saved.result.as_ref().is_none_or(|r| r.sheets.is_empty()) {
            return Err(bad("Create a layout first"));
        }
        ws.busy = true;
        ws.status = "Preparing download…".into();
    }
    let worker = w.clone();
    let kind2 = kind.clone();
    let export_owner = app.auth.principal(&headers)?.user_id;
    let result = blocking(move || {
        let (saved, model, pieces, refs, tabs, dir) = {
            let ws = worker.lock().unwrap();
            let (p, r, _) = ws.expanded();
            (
                ws.saved.clone(),
                ws.model.clone(),
                p,
                r,
                ws.tab_map(),
                ws.dir.clone(),
            )
        };
        let result = saved.result.as_ref().unwrap();
        if saved.sheet_thicknesses.len() != result.sheets.len() {
            return Err(bad("Panelize materials before exporting"));
        }
        let filename = |index: usize, extension: &str| {
            format!(
                "{}{}mm/panel_{:03}.{extension}",
                if saved.settings.cam.material.is_empty()
                    && saved.settings.cam.part_materials.is_empty()
                {
                    String::new()
                } else {
                    format!("{}/", {
                        let m = cam::panel_material(&saved, &result.sheets[index]);
                        if m.is_empty() {
                            "unspecified".into()
                        } else {
                            m
                        }
                    })
                },
                saved.sheet_thicknesses[index],
                index + 1
            )
        };
        let s = &saved.settings;
        let mut files = vec![];
        if kind2 == "step" {
            let model = model.ok_or_else(|| bad("Upload STEP first"))?;
            let out = dir.join(format!("export-{}", Uuid::new_v4()));
            std::fs::create_dir(&out).map_err(anyhow::Error::from)?;
            let exported = {
                let _lock = app.occt.lock().unwrap();
                thetta_step::export_all_sheets(
                    &model,
                    &refs,
                    result,
                    &s.sheet,
                    &out,
                    "nested",
                    Some(&tabs),
                )
            };
            let read = (|| -> Result<(), Error> {
                for (index, path) in exported.map_err(bad)?.into_iter().enumerate() {
                    files.push((
                        filename(index, "step"),
                        std::fs::read(path).map_err(anyhow::Error::from)?,
                    ));
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(out);
            read?;
        } else if s.cam.automatic {
            let db = cam::Catalog::open(&app.root)?;
            let view = db.view(&export_owner, &saved)?;
            let plan = saved
                .cam_plan
                .as_ref()
                .ok_or_else(|| bad("Resolve automatic CAM before export"))?;
            if !plan.ready
                || plan.requires_review && !plan.accepted
                || cam::fingerprint(&saved, &view)? != plan.fingerprint
            {
                return Err(bad(
                    "Automatic cutting plan is unresolved, unaccepted, or stale; resolve it again",
                ));
            }
            for panel in &plan.panels {
                let sheet = &result.sheets[panel.index];
                let profile = panel
                    .cuts
                    .iter()
                    .find(|c| c.operation == "profile")
                    .ok_or_else(|| bad("Profile plan missing"))?;
                let drill = panel.cuts.iter().find(|c| c.operation == "drill");
                let pocket = panel.cuts.iter().find(|c| c.operation == "pocket");
                let mut gc = s.gcode.clone();
                gc.cut_depth = panel.thickness;
                gc.tool_number = Some(profile.tool.number);
                let mut dc = s.drill.clone();
                dc.enabled = drill.is_some();
                dc.tool_number = drill.map(|c| c.tool.number);
                if let Some(c) = drill {
                    dc.peck_depth = c.step_down;
                }
                let mut pc = s.pocket.clone();
                pc.enabled = pocket.is_some();
                pc.tool_number = pocket.map(|c| c.tool.number);
                if let Some(c) = pocket {
                    pc.step_over = c.step_over;
                }
                let code = thetta_cam::generate::generate_sheet_gcode_planned(
                    &thetta_cam::generate::SheetGcodeInput {
                        sheet_index: panel.index,
                        placements: &sheet.placements,
                        pieces: &pieces,
                        tabs: &tabs,
                        gcode_config: &gc,
                        profile_tool: &profile.tool,
                        profile_config: &s.profile,
                        drill_tool: drill.map(|c| &c.tool),
                        drill_config: &dc,
                        pocket_tool: pocket.map(|c| &c.tool),
                        pocket_config: &pc,
                    },
                    profile.step_down,
                    pocket.map_or(profile.step_down, |c| c.step_down),
                    profile.ramp_angle,
                    profile.entry == "waste helix",
                    pocket.map_or(profile.ramp_angle, |c| c.ramp_angle),
                )
                .map_err(bad)?;
                files.push((filename(panel.index, "ngc"), code.into_bytes()));
            }
            files.push((
                "cutting-plan.json".into(),
                serde_json::to_vec_pretty(plan).map_err(anyhow::Error::from)?,
            ));
        } else {
            s.tools.validate_definitions().map_err(bad)?;
            s.gcode.validate().map_err(bad)?;
            s.drill.validate().map_err(bad)?;
            s.pocket.validate().map_err(bad)?;
            let tool = s
                .gcode
                .tool_number
                .and_then(|n| s.tools.by_number(n))
                .ok_or_else(|| bad("Choose an existing profile tool"))?;
            let drill = s.drill.tool_number.and_then(|n| s.tools.by_number(n));
            let pocket = s.pocket.tool_number.and_then(|n| s.tools.by_number(n));
            if s.drill.enabled && drill.is_none() || s.pocket.enabled && pocket.is_none() {
                return Err(bad("Choose tools for enabled operations"));
            }
            for selected in [
                Some(tool),
                drill.filter(|_| s.drill.enabled),
                pocket.filter(|_| s.pocket.enabled),
            ]
            .into_iter()
            .flatten()
            {
                selected.validate().map_err(bad)?;
            }
            for (si, sheet) in result.sheets.iter().enumerate() {
                let mut config = s.gcode.clone();
                config.cut_depth = saved
                    .panel_depth(sheet)
                    .ok_or_else(|| bad("Panel material thickness is missing"))?;
                let code = thetta_cam::generate::generate_sheet_gcode(
                    &thetta_cam::generate::SheetGcodeInput {
                        sheet_index: si,
                        placements: &sheet.placements,
                        pieces: &pieces,
                        tabs: &tabs,
                        gcode_config: &config,
                        profile_tool: tool,
                        profile_config: &s.profile,
                        drill_tool: drill,
                        drill_config: &s.drill,
                        pocket_tool: pocket,
                        pocket_config: &s.pocket,
                    },
                );
                files.push((filename(si, "ngc"), code.into_bytes()));
            }
        }
        zip_files(files).map_err(|e| e.into())
    })
    .await;
    {
        let mut ws = w.lock().unwrap();
        ws.busy = false;
        ws.status = match &result {
            Ok(_) => "Download ready".into(),
            Err(e) => e.1.clone(),
        };
    }
    Ok((
        [
            ("content-type", "application/zip".to_string()),
            (
                "content-disposition",
                format!("attachment; filename=thetta-{kind}.zip"),
            ),
            ("cache-control", "no-store".into()),
        ],
        Bytes::from(result?),
    )
        .into_response())
}
fn router(app: App, static_dir: PathBuf) -> Router {
    let protected = Router::new()
        .route("/api/workspace", post(open).get(snapshot))
        .route(
            "/api/workspaces",
            get(auth::workspaces).post(auth::select_workspace),
        )
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/onshape/import", post(onshape::import))
        .route("/api/onshape/refresh", post(onshape::refresh))
        .route("/api/events", get(events))
        .route("/api/upload", post(upload))
        .route("/api/settings", put(settings))
        .route("/api/cam/catalog", get(cam_api::catalog))
        .route("/api/cam/machine", put(cam_api::machine))
        .route("/api/cam/tools/{number}", put(cam_api::tool))
        .route("/api/cam/recipes", post(cam_api::recipe))
        .route("/api/cam/resolve", post(cam_api::resolve))
        .route("/api/cam/accept", post(cam_api::accept))
        .route("/api/parts/{id}", put(part))
        .route("/api/materials/{thickness}", put(material))
        .route("/api/thickness", put(thickness))
        .route("/api/nesting/start", post(start))
        .route("/api/nesting/stop", post(stop))
        .route("/api/nesting/improve", post(improve))
        .route("/api/tabs", post(tabs))
        .route("/api/export/{kind}", post(export))
        .route("/api/{*rest}", get(|| async { StatusCode::NOT_FOUND }))
        .route_layer(axum::middleware::from_fn_with_state(
            app.clone(),
            auth::gate,
        ));
    Router::new()
        .merge(protected)
        .route(
            "/api/health",
            get(|| async { Json(json!({"status":"ok"})) }),
        )
        .route("/api/auth/status", get(auth::status))
        .route("/api/auth/onshape/start", get(auth::login))
        .route("/api/auth/onshape/callback", get(auth::callback))
        .fallback_service(
            ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(static_dir.join("index.html"))),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .layer(CompressionLayer::new())
        .with_state(app)
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(std::env::var("THETTA_DATA_DIR").unwrap_or(".thetta-data".into()));
    std::fs::create_dir_all(&root)?;
    let auth = Arc::new(auth::Auth::from_env(&root)?);
    let catalog = cam::Catalog::open(&root)?;
    // Migrate account-owned legacy inventories without rewriting project settings.
    for (workspace, owner) in auth.workspace_owners()? {
        if Uuid::parse_str(&workspace).is_err() {
            continue;
        }
        let path = root.join(&workspace).join("workspace.json");
        if path.exists() {
            match std::fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Saved>(&bytes).ok())
            {
                Some(saved) => {
                    catalog.inventory(
                        &owner,
                        if saved.settings.cam.machine_id.is_empty() {
                            "meitill"
                        } else {
                            &saved.settings.cam.machine_id
                        },
                        &saved.settings.tools,
                    )?;
                }
                None => {
                    eprintln!("Skipped unreadable legacy CAM inventory for workspace {workspace}")
                }
            }
        }
    }
    drop(catalog);
    let app = App {
        auth,
        root,
        workspaces: Default::default(),
        occt: Default::default(),
        shutting_down: Default::default(),
    };
    let static_dir = PathBuf::from(std::env::var("THETTA_STATIC_DIR").unwrap_or("web/dist".into()));
    let address = std::env::var("THETTA_BIND").unwrap_or("0.0.0.0:3007".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Thetta listening on {address}");
    let workspaces = app.workspaces.clone();
    let shutting_down = app.shutting_down.clone();
    axum::serve(listener, router(app, static_dir))
        .with_graceful_shutdown(async move {
            #[cfg(unix)]
            {
                let mut term =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("SIGTERM handler");
                tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} }
            }
            #[cfg(not(unix))]
            let _ = tokio::signal::ctrl_c().await;
            shutting_down.store(true, Ordering::Relaxed);
            for workspace in workspaces.lock().unwrap().values() {
                let ws = workspace.lock().unwrap();
                if let Some(cancel) = &ws.cancel {
                    cancel.store(true, Ordering::Relaxed);
                }
                if let Err(e) = ws.persist() {
                    eprintln!("Shutdown save failed: {e}");
                }
            }
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    static OCCT_TEST: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    fn app(root: PathBuf) -> App {
        let auth = Arc::new(
            auth::Auth::new(
                &root.join("auth.sqlite"),
                &[1; 32],
                "https://thetta.finn.xyz".into(),
                "client".into(),
                "secret".into(),
                vec!["owner@example.com".into(), "other@example.com".into()],
            )
            .unwrap(),
        );
        App {
            auth,
            root,
            workspaces: Default::default(),
            occt: Default::default(),
            shutting_down: Default::default(),
        }
    }
    async fn request(
        router: &Router,
        method: &str,
        path: &str,
        cookie: &str,
        body: Option<Value>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header("origin", "https://thetta.finn.xyz");
        if !cookie.is_empty() {
            req = req.header("cookie", cookie);
        }
        let body = if let Some(b) = body {
            req = req.header("content-type", "application/json");
            Body::from(b.to_string())
        } else {
            Body::empty()
        };
        let r = router
            .clone()
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap();
        let status = r.status();
        let headers = r.headers().clone();
        let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec();
        (status, headers, bytes)
    }
    #[tokio::test]
    async fn improve_preserves_layout_tabs_materials_and_persistence() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        assert_eq!(
            request(&r, "POST", "/api/nesting/improve", "", None)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        assert_eq!(
            request(&r, "POST", "/api/nesting/improve", &cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/multi_thickness.step");
        import_bytes(&a, &w, "mixed.step", &std::fs::read(fixture).unwrap(), None).unwrap();
        let (before, tabs) = {
            let mut ws = w.lock().unwrap();
            let jobs = ws.panel_jobs();
            let mut results = vec![];
            for (t, pieces, rotations) in &jobs {
                let mut result = Default::default();
                packing::complete(
                    &mut result,
                    pieces,
                    rotations,
                    &ws.saved.settings.sheet,
                    &Arc::new(AtomicBool::new(false)),
                );
                results.push((*t, result));
            }
            ws.update_panels(&results);
            ws.generate_tabs();
            ws.persist().unwrap();
            (
                serde_json::to_value(&ws.saved.result).unwrap(),
                serde_json::to_value(&ws.saved.tabs).unwrap(),
            )
        };
        // Invalid mode, concurrent work, and stale/partial layouts are rejected atomically.
        {
            w.lock().unwrap().saved.settings.mode = "strip".into();
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/improve", &cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        {
            let mut ws = w.lock().unwrap();
            ws.saved.settings.mode = "sheet".into();
            ws.running = true;
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/improve", &cookie, None)
                .await
                .0,
            StatusCode::CONFLICT
        );
        {
            let mut ws = w.lock().unwrap();
            ws.running = false;
            ws.saved
                .result
                .as_mut()
                .unwrap()
                .unplaced
                .push(thetta_geometry::types::PieceId(999));
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/improve", &cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        {
            w.lock()
                .unwrap()
                .saved
                .result
                .as_mut()
                .unwrap()
                .unplaced
                .clear();
        }
        let _ = start_improvement(a.clone(), headers.clone(), Duration::ZERO)
            .await
            .unwrap();
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        {
            let ws = w.lock().unwrap();
            assert!(!ws.running);
            assert_eq!(ws.status, "No better layout found");
            assert_eq!(serde_json::to_value(&ws.saved.result).unwrap(), before);
            assert_eq!(serde_json::to_value(&ws.saved.tabs).unwrap(), tabs);
        }
        // Real improvement execution keeps thickness groups distinct and all pieces present.
        let _ = start_improvement(a.clone(), headers, Duration::from_millis(100))
            .await
            .unwrap();
        request(&r, "POST", "/api/nesting/stop", &cookie, None).await;
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let ws = w.lock().unwrap();
        assert!(!ws.running);
        let result = ws.saved.result.as_ref().unwrap();
        assert_eq!(result.sheets.len(), ws.saved.sheet_thicknesses.len());
        for (t, pieces, rotations) in ws.panel_jobs() {
            let group = thetta_geometry::types::NestingResult {
                sheets: result
                    .sheets
                    .iter()
                    .zip(&ws.saved.sheet_thicknesses)
                    .filter(|(_, depth)| thickness_key(**depth) == thickness_key(t))
                    .map(|(s, _)| s.clone())
                    .collect(),
                unplaced: vec![],
            };
            thetta_nesting::improve::validate(
                &group,
                &pieces,
                &rotations,
                &ws.saved.settings.sheet,
            )
            .unwrap();
        }
        let persisted: Saved =
            serde_json::from_slice(&std::fs::read(ws.dir.join("workspace.json")).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&persisted.result).unwrap(),
            serde_json::to_value(result).unwrap()
        );
    }
    #[tokio::test]
    async fn accepted_improvement_clears_tabs_and_saves_new_placements() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/single_box_20x10x2.step");
        import_bytes(&a, &w, "box.step", &std::fs::read(fixture).unwrap(), None).unwrap();
        {
            let mut ws = w.lock().unwrap();
            ws.saved.parts[0].piece.quantity = 2;
            ws.saved.parts[0].rotation = "fixed".into();
            let jobs = ws.panel_jobs();
            let (t, pieces, rotations) = &jobs[0];
            let mut result = thetta_geometry::types::NestingResult::default();
            for (piece, rotation) in pieces.iter().zip(rotations) {
                let mut one = Default::default();
                packing::complete(
                    &mut one,
                    std::slice::from_ref(piece),
                    std::slice::from_ref(rotation),
                    &ws.saved.settings.sheet,
                    &Arc::new(AtomicBool::new(false)),
                );
                result.sheets.extend(one.sheets);
            }
            ws.update_panels(&[(*t, result)]);
            ws.saved.tabs.push(TabSet {
                sheet: 0,
                placement: 0,
                tabs: vec![],
            });
            ws.persist().unwrap();
        }
        let _ = start_improvement(a.clone(), headers, Duration::from_millis(200))
            .await
            .unwrap();
        for _ in 0..200 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let ws = w.lock().unwrap();
        assert!(!ws.running);
        assert_eq!(ws.status, "Layout improved");
        assert_eq!(ws.saved.result.as_ref().unwrap().sheets.len(), 1);
        assert!(ws.saved.tabs.is_empty());
        let persisted: Saved =
            serde_json::from_slice(&std::fs::read(ws.dir.join("workspace.json")).unwrap()).unwrap();
        assert!(persisted.tabs.is_empty());
        assert_eq!(persisted.result.unwrap().sheets.len(), 1);
        drop(ws);
        // Older saved layouts can have less clearance than the configured spacing.
        {
            let mut ws = w.lock().unwrap();
            let jobs = ws.panel_jobs();
            let (t, pieces, rotations) = &jobs[0];
            let mut legacy = Default::default();
            let mut old_stock = ws.saved.settings.sheet.clone();
            old_stock.separation = 2.;
            packing::complete(
                &mut legacy,
                pieces,
                rotations,
                &old_stock,
                &Arc::new(AtomicBool::new(false)),
            );
            assert!(
                thetta_nesting::improve::validate(
                    &legacy,
                    pieces,
                    rotations,
                    &ws.saved.settings.sheet
                )
                .is_err()
            );
            ws.update_panels(&[(*t, legacy)]);
            ws.saved.tabs.push(TabSet {
                sheet: 0,
                placement: 0,
                tabs: vec![],
            });
        }
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let _ = start_improvement(a.clone(), headers, Duration::from_millis(200))
            .await
            .unwrap();
        for _ in 0..200 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let ws = w.lock().unwrap();
        assert!(!ws.running);
        assert!(ws.saved.tabs.is_empty());
        let jobs = ws.panel_jobs();
        let (_, pieces, rotations) = &jobs[0];
        thetta_nesting::improve::validate(
            ws.saved.result.as_ref().unwrap(),
            pieces,
            rotations,
            &ws.saved.settings.sheet,
        )
        .unwrap();
    }
    #[tokio::test]
    #[ignore = "requires THETTA_REGRESSION_STEP and THETTA_REGRESSION_WORKSPACE; runs real ordinary panelization"]
    async fn ordinary_panelize_rebuilds_project_without_saved_placements() {
        let _lock = OCCT_TEST.lock().await;
        let source = std::fs::read(std::env::var("THETTA_REGRESSION_STEP").unwrap()).unwrap();
        let saved: Saved = serde_json::from_slice(
            &std::fs::read(std::env::var("THETTA_REGRESSION_WORKSPACE").unwrap()).unwrap(),
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        import_bytes(&a, &w, "regression.step", &source, None).unwrap();
        {
            let mut ws = w.lock().unwrap();
            assert_eq!(ws.saved.parts.len(), saved.parts.len());
            for p in &mut ws.saved.parts {
                let old = saved
                    .parts
                    .iter()
                    .find(|x| x.piece.id == p.piece.id)
                    .unwrap();
                p.enabled = old.enabled;
                p.piece.quantity = old.piece.quantity;
                p.rotation = old.rotation.clone();
            }
            ws.saved.settings = saved.settings.clone();
            ws.saved.excluded_thicknesses = saved.excluded_thicknesses.clone();
            ws.invalidate();
            assert!(ws.saved.result.is_none());
            ws.persist().unwrap();
        }
        let began = Instant::now();
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", &cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        let mut seen_automatic = false;
        let mut last_status = String::new();
        loop {
            {
                let ws = w.lock().unwrap();
                if ws.status != last_status {
                    println!("{:.1}s {}", began.elapsed().as_secs_f64(), ws.status);
                    last_status = ws.status.clone();
                }
                seen_automatic |= ws.status.contains("Consolidating");
                if !ws.running {
                    break;
                }
            }
            assert!(
                began.elapsed() < Duration::from_secs(90),
                "Ordinary panelization must finish within its budget"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let (_, _, body) = request(&r, "GET", "/api/workspace", &cookie, None).await;
        let snapshot: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(snapshot["running"], false);
        let ws = w.lock().unwrap();
        let result = ws.saved.result.as_ref().unwrap();
        let count: usize = result.sheets.iter().map(|s| s.placements.len()).sum();
        assert!(
            seen_automatic,
            "The ordinary start endpoint must run consolidation automatically"
        );
        if let Ok(path) = std::env::var("THETTA_REGRESSION_OUTPUT") {
            std::fs::write(path, serde_json::to_vec(&ws.saved).unwrap()).unwrap();
        }
        for (n, sheet) in result.sheets.iter().enumerate() {
            println!(
                "Panel {}: {:?}",
                n + 1,
                sheet
                    .placements
                    .iter()
                    .map(|p| (p.piece_id.0 - 1) / 1001 + 1)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(count, 37);
        assert!(result.unplaced.is_empty());
        assert!(
            result.sheets.len() <= 7,
            "Expected automatic consolidation to at most seven sheets; got {}",
            result.sheets.len()
        );
        for (t, pieces, rotations) in ws.panel_jobs() {
            let group = thetta_geometry::types::NestingResult {
                sheets: result
                    .sheets
                    .iter()
                    .zip(&ws.saved.sheet_thicknesses)
                    .filter(|(_, depth)| thickness_key(**depth) == thickness_key(t))
                    .map(|(s, _)| s.clone())
                    .collect(),
                unplaced: vec![],
            };
            thetta_nesting::improve::validate(
                &group,
                &pieces,
                &rotations,
                &ws.saved.settings.sheet,
            )
            .unwrap();
        }
        let persisted: Saved =
            serde_json::from_slice(&std::fs::read(ws.dir.join("workspace.json")).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(persisted.result).unwrap(),
            serde_json::to_value(result).unwrap()
        );
        println!(
            "Normal Panelize generated {count} parts on {} sheets in {:.1}s, with no saved placements or Improve request",
            result.sheets.len(),
            began.elapsed().as_secs_f64()
        );
    }
    #[tokio::test]
    async fn isolation_validation_and_restart_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        assert_eq!(
            request(&r, "GET", "/api/workspace", "", None).await.0,
            StatusCode::UNAUTHORIZED
        );
        let login = a.auth.test_login(dir.path(), "owner@example.com");
        let cookie = login.as_str();
        let (_, _, b) = request(&r, "POST", "/api/workspace", cookie, None).await;
        let mut s: Value =
            serde_json::from_slice::<Value>(&b).unwrap()["workspace"]["settings"].clone();
        s["sheet"]["width"] = json!(-1);
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(s.clone()))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        s["sheet"]["width"] = json!(500);
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(s)).await.0,
            StatusCode::OK
        );
        let other_login = a.auth.test_login(dir.path(), "other@example.com");
        let (_, _, other) = request(&r, "POST", "/api/workspace", &other_login, None).await;
        assert_eq!(
            serde_json::from_slice::<Value>(&other).unwrap()["workspace"]["settings"]["sheet"]["width"],
            2440.
        );
        let r2 = router(app(dir.path().into()), PathBuf::from("web/dist"));
        let (_, _, restored) = request(&r2, "POST", "/api/workspace", cookie, None).await;
        assert_eq!(
            serde_json::from_slice::<Value>(&restored).unwrap()["workspace"]["settings"]["sheet"]["width"],
            500.
        );
        assert_eq!(
            request(&r2, "POST", "/api/nesting/start", cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(&r2, "POST", "/api/export/step", cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(
                &r2,
                "GET",
                "/api/workspace",
                "__Host-thetta_session=../../etc",
                None
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn material_group_enable_migrates_legacy_exclusion_without_enabling_other_materials() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/single_box_20x10x2.step");
        import_bytes(&a, &w, "box.step", &std::fs::read(fixture).unwrap(), None).unwrap();
        {
            let mut ws = w.lock().unwrap();
            let mut duplicate = ws.saved.parts[0].clone();
            duplicate.piece.id = thetta_geometry::types::PieceId(1);
            ws.saved.parts.push(duplicate);
            ws.saved.settings.cam.material = "plywood".into();
            ws.saved
                .settings
                .cam
                .part_materials
                .insert("1".into(), "mdf".into());
            ws.saved.excluded_thicknesses = vec![2.];
        }
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":true,"material":"plywood"}))
            )
            .await
            .0,
            StatusCode::OK
        );
        {
            let ws = w.lock().unwrap();
            assert!(ws.saved.excluded_thicknesses.is_empty());
            assert!(ws.saved.includes(&ws.saved.parts[0]));
            assert!(!ws.saved.includes(&ws.saved.parts[1]));
        }
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":true,"material":"mdf"}))
            )
            .await
            .0,
            StatusCode::OK
        );
        let ws = w.lock().unwrap();
        assert!(ws.saved.parts.iter().all(|p| ws.saved.includes(p)));
    }
    #[tokio::test]
    async fn material_selection_is_atomic_persistent_and_preserves_individual_choices() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/multi_thickness.step");
        import_bytes(&a, &w, "mixed.step", &std::fs::read(fixture).unwrap(), None).unwrap();
        let id = {
            let mut ws = w.lock().unwrap();
            let p = ws
                .saved
                .parts
                .iter_mut()
                .find(|p| p.thickness_mm == Some(2.))
                .unwrap();
            p.enabled = false;
            p.piece.quantity = 3;
            p.rotation = "fixed".into();
            let id = p.piece.id;
            ws.saved.result = Some(Default::default());
            ws.saved.settings.optimizer.time_limit_secs = 0.1;
            ws.persist().unwrap();
            id
        };
        // Repeating an already enabled selection must not invalidate a layout.
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":true}))
            )
            .await
            .0,
            StatusCode::OK
        );
        assert!(w.lock().unwrap().saved.result.is_some());
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/materials/2",
                "",
                Some(json!({"enabled":false}))
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":false}))
            )
            .await
            .0,
            StatusCode::OK
        );
        {
            let ws = w.lock().unwrap();
            assert_eq!(ws.saved.excluded_thicknesses, vec![2.]);
            assert!(ws.saved.result.is_none());
            assert_eq!(
                ws.panel_jobs()
                    .iter()
                    .map(|(t, _, _)| *t)
                    .collect::<Vec<_>>(),
                vec![5.]
            );
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", &cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(!w.lock().unwrap().running);
        assert_eq!(w.lock().unwrap().saved.sheet_thicknesses, vec![5.]);
        let (status, _, bytes) = request(&r, "POST", "/api/export/step", &cookie, None).await;
        assert_eq!(status, StatusCode::OK);
        let mut exported = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(exported.len(), 1);
        assert!(exported.by_index(0).unwrap().name().starts_with("5mm/"));
        let restarted = app(dir.path().into());
        let r2 = router(restarted.clone(), PathBuf::from("web/dist"));
        request(&r2, "POST", "/api/workspace", &cookie, None).await;
        let restored = session(&restarted, &headers).unwrap();
        assert_eq!(
            restored.lock().unwrap().saved.excluded_thicknesses,
            vec![2.]
        );
        assert_eq!(
            request(
                &r2,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":true}))
            )
            .await
            .0,
            StatusCode::OK
        );
        {
            let ws = restored.lock().unwrap();
            let p = ws.saved.parts.iter().find(|p| p.piece.id == id).unwrap();
            assert!(!p.enabled);
            assert_eq!(p.piece.quantity, 3);
            assert_eq!(p.rotation, "fixed");
            assert_eq!(
                ws.panel_jobs()
                    .iter()
                    .map(|(t, _, _)| *t)
                    .collect::<Vec<_>>(),
                vec![2., 5.]
            );
        }
        for value in ["3", "40", "NaN"] {
            assert_eq!(
                request(
                    &r2,
                    "PUT",
                    &format!("/api/materials/{value}"),
                    &cookie,
                    Some(json!({"enabled":false}))
                )
                .await
                .0,
                StatusCode::BAD_REQUEST
            );
        }
        restored.lock().unwrap().running = true;
        assert_eq!(
            request(
                &r2,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":false}))
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
        restored.lock().unwrap().running = false;
        assert_eq!(
            request(
                &r2,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":false}))
            )
            .await
            .0,
            StatusCode::OK
        );
        {
            let mut ws = restored.lock().unwrap();
            for p in &mut ws.saved.parts {
                if p.thickness_mm == Some(2.) && p.enabled {
                    p.piece.quantity = 1000;
                }
            }
        }
        assert_eq!(
            request(
                &r2,
                "PUT",
                "/api/materials/2",
                &cookie,
                Some(json!({"enabled":true}))
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            restored.lock().unwrap().saved.excluded_thicknesses,
            vec![2.]
        );
    }
    #[tokio::test]
    async fn mixed_thicknesses_get_separate_panels_and_material_specific_exports() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let w = session(
            &a,
            &HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]),
        )
        .unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/multi_thickness.step");
        import_bytes(&a, &w, "mixed.step", &std::fs::read(fixture).unwrap(), None).unwrap();
        {
            let mut ws = w.lock().unwrap();
            assert_eq!(ws.saved.parts.len(), 4);
            assert!(ws.saved.parts.iter().all(|p| p.thickness_mm.is_some()));
            ws.saved.settings.sheet = thetta_geometry::types::SheetConfig {
                width: 100.,
                height: 100.,
                separation: 2.,
            };
            ws.saved.settings.optimizer.time_limit_secs = 0.2;
            ws.saved.settings.gcode.cut_depth = 99.;
            ws.saved.settings.gcode.tool_number = Some(1);
            ws.saved.settings.drill.enabled = false;
            ws.saved.settings.pocket.enabled = false;
            ws.saved.settings.tools=serde_json::from_value(json!({"tools":[{"number":1,"name":"6mm","diameter":6,"flute_count":2,"feed_rate":1500,"plunge_rate":500,"spindle_speed":18000}]})).unwrap();
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", &cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        {
            let ws = w.lock().unwrap();
            assert!(!ws.running);
            let result = ws.saved.result.as_ref().unwrap();
            assert!(result.unplaced.is_empty());
            assert_eq!(
                result
                    .sheets
                    .iter()
                    .map(|s| s.placements.len())
                    .sum::<usize>(),
                4
            );
            assert_eq!(ws.saved.sheet_thicknesses, vec![2., 5.]);
            for (sheet, thickness) in result.sheets.iter().zip(&ws.saved.sheet_thicknesses) {
                for placement in &sheet.placements {
                    let parent = (placement.piece_id.0 - 1) / 1001;
                    let part = ws
                        .saved
                        .parts
                        .iter()
                        .find(|p| p.piece.id.0 == parent)
                        .unwrap();
                    assert_eq!(
                        thickness_key(part.thickness_mm.unwrap()) as f64 * THICKNESS_BUCKET_MM,
                        *thickness
                    );
                }
            }
        }
        for kind in ["step", "gcode"] {
            let (status, _, data) =
                request(&r, "POST", &format!("/api/export/{kind}"), &cookie, None).await;
            assert_eq!(status, StatusCode::OK);
            let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();
            assert_eq!(zip.len(), 2);
            for i in 0..zip.len() {
                let mut file = zip.by_index(i).unwrap();
                let thickness = file
                    .name()
                    .split("mm/")
                    .next()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap();
                assert!([2., 5.].contains(&thickness));
                let mut bytes = vec![];
                std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();
                if kind == "gcode" {
                    let text = String::from_utf8(bytes).unwrap();
                    assert!(
                        text.contains(&format!("Z-{thickness:.3}")),
                        "cut depth must follow the panel's material"
                    );
                    assert!(!text.contains("Z-99.000"));
                } else {
                    let path = dir.path().join(format!("material-{i}.step"));
                    std::fs::write(&path, bytes).unwrap();
                    let imported = import_step(&path, None, &ExtractionConfig::default()).unwrap();
                    let (_, refs) = extract_all_pieces(&imported.model).unwrap();
                    // The existing STEP exporter also includes a 10 mm CNC bed solid.
                    assert!(
                        refs.iter()
                            .any(|r| (r.thickness_mm - thickness).abs() < 1e-4)
                    );
                    assert!(
                        refs.iter()
                            .all(|r| (r.thickness_mm - thickness).abs() < 1e-4
                                || (r.thickness_mm - 10.).abs() < 1e-4)
                    );
                }
            }
        }
        let restored = app(dir.path().into());
        let loaded = load_workspace(
            &restored,
            &a.auth
                .principal(&HeaderMap::from_iter([(
                    "cookie".parse().unwrap(),
                    cookie.parse().unwrap(),
                )]))
                .unwrap()
                .workspace,
        )
        .unwrap();
        assert_eq!(loaded.lock().unwrap().saved.sheet_thicknesses, vec![2., 5.]);
        {
            let mut ws = w.lock().unwrap();
            ws.saved.settings.sheet.width = 1.;
            ws.saved.settings.sheet.height = 1.;
            ws.saved.settings.sheet.separation = 0.;
            ws.invalidate();
        }
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", &cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let ws = w.lock().unwrap();
        assert!(!ws.running);
        assert_eq!(ws.saved.result.as_ref().unwrap().unplaced.len(), 4);
        assert!(ws.saved.result.as_ref().unwrap().sheets.is_empty());
        assert!(ws.saved.sheet_thicknesses.is_empty());
    }
    #[tokio::test]
    async fn real_step_quantities_nesting_tabs_and_exports() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let login = a.auth.test_login(dir.path(), "owner@example.com");
        let cookie = login.as_str();
        request(&r, "POST", "/api/workspace", cookie, None).await;
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_data/step/single_box_20x10x2.step");
        let mut body=b"--thetta-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"box.step\"\r\nContent-Type: application/octet-stream\r\n\r\n".to_vec();
        body.extend(std::fs::read(fixture).unwrap());
        body.extend(b"\r\n--thetta-test--\r\n");
        let res = r
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/upload")
                    .header("cookie", cookie)
                    .header("origin", "https://thetta.finn.xyz")
                    .header("content-type", "multipart/form-data; boundary=thetta-test")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let (_, _, b) = request(&r, "GET", "/api/workspace", cookie, None).await;
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["workspace"]["parts"].as_array().unwrap().len(), 1);
        assert!(
            !String::from_utf8(b)
                .unwrap()
                .contains(dir.path().to_str().unwrap())
        );
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/parts/0",
                cookie,
                Some(json!({"quantity":3,"enabled":true,"rotation":"fixed"}))
            )
            .await
            .0,
            StatusCode::OK
        );
        let mut s = v["workspace"]["settings"].clone();
        s["sheet"] = json!({"width":100,"height":100,"separation":2});
        s["optimizer"]["time_limit_secs"] = json!(0.2);
        s["tools"] = json!({"tools":[
            {"number":1,"name":"6mm","diameter":6,"flute_count":2,"feed_rate":1500,"plunge_rate":500,"spindle_speed":18000},
            {"number":99,"name":"Imported probe","diameter":0,"flute_count":0,"feed_rate":0,"plunge_rate":0,"spindle_speed":0}
        ]});
        s["tabs"]["auto_size"] = json!(false);
        s["tabs"]["tab_width"] = json!(2);
        s["tabs"]["tab_height"] = json!(1);
        s["tabs"]["material_thickness"] = json!(2);
        s["tabs"]["corner_exclusion"] = json!(1);
        s["tabs"]["tab_distance"] = json!(10);
        s["gcode"]["tool_number"] = json!(1);
        s["gcode"]["cut_depth"] = json!(2);
        s["drill"]["enabled"] = json!(false);
        s["pocket"]["enabled"] = json!(false);
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(s.clone()))
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(s.clone()))
                .await
                .0,
            StatusCode::CONFLICT
        );
        let mut done = false;
        for _ in 0..100 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let (_, _, b) = request(&r, "GET", "/api/workspace", cookie, None).await;
            let v: Value = serde_json::from_slice(&b).unwrap();
            if v["running"] == false {
                assert_eq!(
                    v["workspace"]["result"]["sheets"][0]["placements"]
                        .as_array()
                        .unwrap()
                        .len(),
                    3
                );
                done = true;
                break;
            }
        }
        assert!(done, "optimizer must stop within its budget");
        assert_eq!(
            request(&r, "POST", "/api/tabs", cookie, None).await.0,
            StatusCode::OK
        );
        let (_, _, b) = request(&r, "GET", "/api/workspace", cookie, None).await;
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert!(
            v["workspace"]["tabs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| !t["tabs"].as_array().unwrap().is_empty())
        );
        // Library definitions may be incomplete, but every selected cutting
        // parameter is required before any G-code can be downloaded.
        for field in [
            "diameter",
            "flute_count",
            "feed_rate",
            "plunge_rate",
            "spindle_speed",
        ] {
            let mut pending = s.clone();
            pending["tools"]["tools"][0][field] = json!(0);
            assert_eq!(
                request(&r, "PUT", "/api/settings", cookie, Some(pending))
                    .await
                    .0,
                StatusCode::OK
            );
            assert_eq!(
                request(&r, "POST", "/api/export/gcode", cookie, None)
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        let mut invalid = s.clone();
        invalid["tools"]["tools"][0]["feed_rate"] = json!(-1);
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(invalid))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        let mut duplicate = s.clone();
        duplicate["tools"]["tools"][1]["number"] = json!(1);
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(duplicate))
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(&r, "PUT", "/api/settings", cookie, Some(s)).await.0,
            StatusCode::OK
        );
        for kind in ["step", "gcode"] {
            let (status, _, zip) =
                request(&r, "POST", &format!("/api/export/{kind}"), cookie, None).await;
            assert_eq!(status, StatusCode::OK);
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
            assert_eq!(archive.len(), 1);
            let mut file = archive.by_index(0).unwrap();
            let mut data = vec![];
            std::io::Read::read_to_end(&mut file, &mut data).unwrap();
            if kind == "step" {
                let output = dir.path().join("reimport.step");
                std::fs::write(&output, data).unwrap();
                let _lock = a.occt.lock().unwrap();
                let imported = import_step(&output, None, &ExtractionConfig::default()).unwrap();
                assert_eq!(imported.pieces.len(), 3);
            } else {
                let code = String::from_utf8(data).unwrap();
                assert!(code.contains("G54"));
                assert!(code.contains("M30") || code.contains("M2"));
            }
        }
        let r2 = router(app(dir.path().into()), PathBuf::from("web/dist"));
        let (_, _, b) = request(&r2, "POST", "/api/workspace", cookie, None).await;
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["pieces"].as_array().unwrap().len(), 3);
        assert_eq!(
            v["workspace"]["settings"]["tools"]["tools"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            v["workspace"]["settings"]["tools"]["tools"][1]["spindle_speed"],
            0
        );
        assert_eq!(
            v["workspace"]["result"]["sheets"][0]["placements"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            request(&r2, "POST", "/api/nesting/start", cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&r2, "POST", "/api/nesting/stop", cookie, None)
                .await
                .0,
            StatusCode::OK
        );
    }
    #[tokio::test]
    async fn onshape_import_refresh_and_failure_preserve_the_accepted_source() {
        let _lock = OCCT_TEST.lock().await;
        let root = tempfile::tempdir().unwrap();
        let mut app = app(root.path().into());
        let mode = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mode2 = mode.clone();
        let fixture = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test_data/step/single_box_20x10x2.step"),
        )
        .unwrap();
        let provider=Router::new().fallback(move |request:axum::extract::Request| {
            let mode=mode2.clone();let fixture=fixture.clone();
            async move {
                assert_eq!(request.headers()["authorization"],"Bearer test-access");
                let path=request.uri().path().to_string();let current=mode.load(Ordering::Relaxed);
                if path.ends_with("/elements") {return Json(json!([{"id":"0123456789abcdef01234567","name":"Fixture","elementType":if current==0 {"PARTSTUDIO"}else{"ASSEMBLY"}}])).into_response();}
                if path.ends_with("/translations") {
                    assert!(path.starts_with(if current==0 {"/api/v10/partstudios/"}else{"/api/v10/assemblies/"}));
                    let body=to_bytes(request.into_body(),100000).await.unwrap();let body:Value=serde_json::from_slice(&body).unwrap();
                    assert_eq!(body["formatName"],"STEP");assert_eq!(body["storeInDocument"],false);assert_eq!(body["configuration"],"size=small");
                    return Json(json!({"id":"translation-id","requestState":"DONE","resultExternalDataIds":["export-id"]})).into_response();
                }
                if path.ends_with("/externaldata/export-id") {
                    if current==2{return StatusCode::PAYMENT_REQUIRED.into_response();}
                    return ([("content-type","application/step")],fixture).into_response();
                }
                panic!("Unexpected Onshape API call: {path}")
            }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        Arc::get_mut(&mut app.auth).unwrap().api_root =
            format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move { axum::serve(listener, provider).await.unwrap() });
        let login = app.auth.test_login(root.path(), "owner@example.com");
        let r = router(app.clone(), PathBuf::from("web/dist"));
        request(&r, "POST", "/api/workspace", &login, None).await;
        let source = "https://cad.onshape.com/documents/0123456789abcdef01234567/w/0123456789abcdef01234567/e/0123456789abcdef01234567?configuration=size%3Dsmall";
        for version in [false, true] {
            if version {
                mode.store(1, Ordering::Relaxed);
            }
            let url = if version {
                source.replace("/w/", "/v/")
            } else {
                source.into()
            };
            assert_eq!(
                request(
                    &r,
                    "POST",
                    "/api/onshape/import",
                    &login,
                    Some(json!({"url":url}))
                )
                .await
                .0,
                StatusCode::OK
            );
            let mut done = false;
            for _ in 0..100 {
                tokio::time::sleep(Duration::from_millis(20)).await;
                let (_, _, body) = request(&r, "GET", "/api/workspace", &login, None).await;
                let snap: Value = serde_json::from_slice(&body).unwrap();
                if snap["busy"] == false {
                    assert_eq!(snap["workspace"]["parts"].as_array().unwrap().len(), 1);
                    assert!(
                        snap["workspace"]["onshape"]["url"]
                            .as_str()
                            .unwrap()
                            .contains(if version { "/v/" } else { "/w/" })
                    );
                    done = true;
                    break;
                }
            }
            assert!(done);
        }
        let ws = session(
            &app,
            &HeaderMap::from_iter([("cookie".parse().unwrap(), login.parse().unwrap())]),
        )
        .unwrap();
        let before = serde_json::to_value(&ws.lock().unwrap().saved).unwrap();
        mode.store(2, Ordering::Relaxed);
        assert_eq!(
            request(&r, "POST", "/api/onshape/refresh", &login, None)
                .await
                .0,
            StatusCode::OK
        );
        let mut done = false;
        for _ in 0..100 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            if !ws.lock().unwrap().busy {
                done = true;
                break;
            }
        }
        assert!(done);
        assert!(ws.lock().unwrap().status.contains("allowance"));
        assert_eq!(
            serde_json::to_value(&ws.lock().unwrap().saved).unwrap(),
            before
        );
        handle.abort();
    }
    #[tokio::test]
    async fn automatic_cam_api_review_export_and_catalog_isolation() {
        let _lock = OCCT_TEST.lock().await;
        let dir = tempfile::tempdir().unwrap();
        let a = app(dir.path().into());
        let r = router(a.clone(), PathBuf::from("web/dist"));
        let cookie = a.auth.test_login(dir.path(), "owner@example.com");
        assert_eq!(
            request(&r, "GET", "/api/cam/catalog", "", None).await.0,
            StatusCode::UNAUTHORIZED
        );
        request(&r, "POST", "/api/workspace", &cookie, None).await;
        let headers = HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())]);
        let w = session(&a, &headers).unwrap();
        let source = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test_data/step/single_box_20x10x2.step"),
        )
        .unwrap();
        import_bytes(&a, &w, "box.step", &source, None).unwrap();
        let settings = {
            let mut ws = w.lock().unwrap();
            let mut second = ws.saved.parts[0].clone();
            second.piece.id = thetta_geometry::types::PieceId(1);
            ws.saved.parts.push(second);
            let mut reference = ws.refs[0].clone();
            reference.piece_id = thetta_geometry::types::PieceId(1);
            ws.refs.push(reference);
            let mut s = ws.saved.settings.clone();
            s.sheet.width = 100.;
            s.sheet.height = 100.;
            s.sheet.separation = 20.;
            s.optimizer.time_limit_secs = 0.2;
            s.drill.enabled = false;
            s.pocket.enabled = false;
            s.cam.automatic = true;
            s.cam.material = "plywood".into();
            s.cam.part_materials.insert("1".into(), "mdf".into());
            s.tools=serde_json::from_value(json!({"tools":[{"number":5,"name":"46216-K","diameter":12.7,"flute_count":3,"feed_rate":0,"plunge_rate":0,"spindle_speed":0}]})).unwrap();
            s
        };
        assert_eq!(
            request(&r, "PUT", "/api/settings", &cookie, Some(json!(settings)))
                .await
                .0,
            StatusCode::OK
        );
        let (_, _, b) = request(&r, "GET", "/api/cam/catalog", &cookie, None).await;
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["tools"].as_array().unwrap().len(), 1);
        let mut machine = v["machine"].clone();
        machine["confirmed"] = json!(true);
        machine["rpm_min"] = json!(6000);
        machine["feed_max"] = json!(12000);
        machine["plunge_max"] = json!(3000);
        machine["step_down_max"] = json!(1);
        machine["workholding"] = json!("clamps");
        machine["chip_removal"] = json!("extraction");
        assert_eq!(
            request(
                &r,
                "PUT",
                "/api/cam/machine",
                &cookie,
                Some(machine.clone())
            )
            .await
            .0,
            StatusCode::OK
        );
        let mut tool = v["tools"][0].clone();
        tool["geometry_confirmed"] = json!(true);
        tool["cutting_length"] = json!(38.1);
        tool["rpm_max"] = json!(24000);
        assert_eq!(
            request(&r, "PUT", "/api/cam/tools/5", &cookie, Some(tool))
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&r, "POST", "/api/nesting/start", &cookie, None)
                .await
                .0,
            StatusCode::OK
        );
        for _ in 0..100 {
            if !w.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        {
            let ws = w.lock().unwrap();
            assert!(!ws.running);
            assert_eq!(ws.saved.result.as_ref().unwrap().sheets.len(), 2);
            assert_eq!(ws.saved.sheet_materials, vec!["mdf", "plywood"]);
        }
        let (status, _, body) = request(&r, "POST", "/api/cam/resolve", &cookie, None).await;
        assert_eq!(status, StatusCode::OK);
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            v["workspace"]["cam_plan"]["ready"], true,
            "{}",
            v["workspace"]["cam_plan"]
        );
        assert_eq!(
            request(&r, "POST", "/api/export/gcode", &cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        let fingerprint = v["workspace"]["cam_plan"]["fingerprint"].clone();
        assert_eq!(
            request(
                &r,
                "POST",
                "/api/cam/accept",
                &cookie,
                Some(json!({"fingerprint":"stale"}))
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(
                &r,
                "POST",
                "/api/cam/accept",
                &cookie,
                Some(json!({"fingerprint":fingerprint}))
            )
            .await
            .0,
            StatusCode::OK
        );
        let (status, _, zip) = request(&r, "POST", "/api/export/gcode", &cookie, None).await;
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&zip));
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        assert_eq!(archive.len(), 3);
        for name in ["mdf/2mm/panel_001.ngc", "plywood/2mm/panel_002.ngc"] {
            let mut file = archive.by_name(name).unwrap();
            let mut code = String::new();
            std::io::Read::read_to_string(&mut file, &mut code).unwrap();
            assert!(code.contains("G43 H5"));
            assert!(code.contains("Z-1.000"));
            assert!(code.contains("Z-2.000"));
        }
        assert_eq!(
            request(&r, "PUT", "/api/cam/machine", &cookie, Some(machine))
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&r, "POST", "/api/export/gcode", &cookie, None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        let other = a.auth.test_login(dir.path(), "other@example.com");
        request(&r, "POST", "/api/workspace", &other, None).await;
        let (_, _, b) = request(&r, "GET", "/api/cam/catalog", &other, None).await;
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert!(v["tools"].as_array().unwrap().is_empty());
        assert_eq!(v["machine"]["confirmed"], false);
    }
}
