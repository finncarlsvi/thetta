use crate::cam::{Catalog, Installed, Machine, Recipe};
use crate::*;
pub async fn catalog(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<cam::CatalogView>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let saved = w.lock().unwrap().saved.clone();
    Ok(Json(Catalog::open(&app.root)?.view(&p.user_id, &saved)?))
}
pub async fn machine(
    State(app): State<App>,
    headers: HeaderMap,
    Json(m): Json<Machine>,
) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let mut w = w.lock().unwrap();
    idle(&w)?;
    let m = Catalog::open(&app.root)?
        .save_machine(&p.user_id, m)
        .map_err(bad)?;
    w.saved.settings.cam.machine_id = m.id.clone();
    w.saved.cam_plan = None;
    w.saved.revision += 1;
    w.persist()?;
    Ok(Json(json!(m)))
}
pub async fn tool(
    State(app): State<App>,
    headers: HeaderMap,
    Path(number): Path<u32>,
    Json(t): Json<Installed>,
) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let mut w = w.lock().unwrap();
    idle(&w)?;
    if t.definition.number != number {
        return Err(bad("Tool number does not match"));
    }
    let id = if w.saved.settings.cam.machine_id.is_empty() {
        "meitill"
    } else {
        &w.saved.settings.cam.machine_id
    };
    let t = Catalog::open(&app.root)?
        .save_tool(&p.user_id, id, t)
        .map_err(bad)?;
    if let Some(old) = w
        .saved
        .settings
        .tools
        .tools
        .iter_mut()
        .find(|x| x.number == number)
    {
        *old = t.definition.clone();
    } else {
        w.saved.settings.tools.tools.push(t.definition.clone());
    }
    w.saved.cam_plan = None;
    w.saved.revision += 1;
    w.persist()?;
    Ok(Json(json!(t)))
}
pub async fn recipe(
    State(app): State<App>,
    headers: HeaderMap,
    Json(r): Json<Recipe>,
) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let mut ws = w.lock().unwrap();
    idle(&ws)?;
    let r = Catalog::open(&app.root)?
        .save_recipe(&p.user_id, r)
        .map_err(bad)?;
    ws.saved.cam_plan = None;
    ws.persist()?;
    Ok(Json(json!(r)))
}
pub async fn resolve(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let saved = {
        let mut ws = w.lock().unwrap();
        idle(&ws)?;
        ws.busy = true;
        ws.status = "Calculating cutting plan…".into();
        ws.saved.clone()
    };
    let root = app.root.clone();
    let previous = saved.cam_plan.clone();
    let result = blocking(move || {
        let db = Catalog::open(&root)?;
        let view = db.view(&p.user_id, &saved)?;
        let mut plan = cam::resolve(&saved, &view)?;
        if let Some(old) = previous {
            if old.fingerprint == plan.fingerprint {
                plan.accepted = old.accepted;
            }
        }
        db.store_plan(&p.user_id, &p.workspace, &plan)?;
        Ok(plan)
    })
    .await;
    let mut ws = w.lock().unwrap();
    ws.busy = false;
    match result {
        Ok(plan) => {
            ws.status = if plan.ready {
                "Cutting plan calculated"
            } else {
                "CAM setup required"
            }
            .into();
            ws.saved.cam_plan = Some(plan);
            ws.persist()?;
            Ok(Json(ws.snapshot()))
        }
        Err(e) => {
            ws.status = e.1.clone();
            Err(e)
        }
    }
}
#[derive(Deserialize)]
pub struct Acceptance {
    fingerprint: String,
}
pub async fn accept(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Acceptance>,
) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let w = session(&app, &headers)?;
    let mut w = w.lock().unwrap();
    idle(&w)?;
    let db = Catalog::open(&app.root)?;
    let view = db.view(&p.user_id, &w.saved)?;
    let fingerprint = cam::fingerprint(&w.saved, &view)?;
    let plan = w
        .saved
        .cam_plan
        .as_mut()
        .ok_or_else(|| bad("Resolve a cutting plan first"))?;
    if !plan.ready || plan.fingerprint != input.fingerprint || fingerprint != input.fingerprint {
        return Err(bad(
            "Cutting plan is unresolved or stale. Resolve it again.",
        ));
    }
    plan.accepted = true;
    db.store_plan(&p.user_id, &p.workspace, plan)?;
    w.persist()?;
    Ok(Json(w.snapshot()))
}
