use super::*;
use auth::{Auth, Principal};
use axum::extract::Extension;
use reqwest::{Method, Url};
const LIMIT: usize = 100 * 1024 * 1024;
#[derive(Debug, Clone)]
struct Source {
    did: String,
    kind: String,
    wid: String,
    eid: String,
    configuration: Option<String>,
    url: String,
}
fn parse(input: &str) -> Result<Source, Error> {
    if input.len() > 16384 {
        return Err(bad("Onshape link is too long"));
    }
    let mut url = Url::parse(input.trim())
        .map_err(|_| bad("Paste an Onshape Part Studio or assembly URL"))?;
    if url.scheme() != "https"
        || url.host_str() != Some("cad.onshape.com")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(bad("Use an HTTPS link from cad.onshape.com"));
    }
    let segments: Vec<_> = url
        .path_segments()
        .ok_or_else(|| bad("Invalid Onshape URL"))?
        .filter(|v| !v.is_empty())
        .collect();
    if segments.len() != 6
        || segments[0] != "documents"
        || !["w", "v"].contains(&segments[2])
        || segments[4] != "e"
    {
        return Err(bad(
            "Link to a Part Studio or assembly tab in a workspace or version",
        ));
    }
    for i in [1, 3, 5] {
        if segments[i].len() != 24 || !segments[i].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(bad("Invalid Onshape document or tab ID"));
        }
    }
    let (did, kind, wid, eid) = (
        segments[1].into(),
        segments[2].into(),
        segments[3].into(),
        segments[5].into(),
    );
    let configuration = url
        .query_pairs()
        .find(|(k, _)| k == "configuration")
        .map(|(_, v)| v.to_string());
    url.set_fragment(None);
    url.set_query(None);
    if let Some(c) = &configuration {
        url.query_pairs_mut().append_pair("configuration", c);
    }
    Ok(Source {
        did,
        kind,
        wid,
        eid,
        configuration,
        url: url.into(),
    })
}
fn api_url(auth: &Auth, path: &[&str]) -> Url {
    let mut url = Url::parse(&auth.api_root).unwrap();
    url.path_segments_mut()
        .unwrap()
        .extend(["api", "v10"].into_iter().chain(path.iter().copied()));
    url
}
async fn request(
    auth: &Auth,
    p: &Principal,
    method: Method,
    mut url: Url,
    body: Option<&Value>,
) -> Result<reqwest::Response, Error> {
    let began = Instant::now();
    let mut refreshed = false;
    let mut redirects = 0;
    let downloading = method == Method::GET && url.path().contains("/externaldata/");
    let mut delay = 2;
    loop {
        let token = auth.access_token(p).await?;
        let vendor = url.host_str() == Url::parse(&auth.api_root).unwrap().host_str()
            || url.host_str().is_some_and(|h| h.ends_with(".onshape.com"));
        let mut req = auth.http.request(method.clone(), url.clone());
        if vendor {
            req = req.bearer_auth(token);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req
            .send()
            .await
            .map_err(|_| bad("Could not reach Onshape. Try again."))?;
        let status = response.status();
        if status.is_redirection() && redirects < 5 {
            let next = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| url.join(v).ok())
                .ok_or_else(|| bad("Invalid redirect from Onshape"))?;
            let same = next.origin() == Url::parse(&auth.api_root).unwrap().origin();
            let onshape = next.scheme() == "https"
                && next.port().is_none()
                && next.host_str().is_some_and(|h| h.ends_with(".onshape.com"));
            let storage = downloading
                && next.scheme() == "https"
                && next.port().is_none()
                && next
                    .host_str()
                    .is_some_and(|h| h.ends_with(".amazonaws.com"));
            if !(same || onshape || storage)
                || !next.username().is_empty()
                || next.password().is_some()
            {
                return Err(bad("Onshape returned an unsupported download redirect"));
            }
            url = next;
            redirects += 1;
            continue;
        }

        if status.is_success() {
            return Ok(response);
        }
        if status == reqwest::StatusCode::UNAUTHORIZED && !refreshed {
            auth.expire_access(&p.user_id)?;
            refreshed = true;
            continue;
        }
        if status == reqwest::StatusCode::UNAUTHORIZED {
            auth.end_sessions(&p.user_id)?;
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS
            && began.elapsed() < Duration::from_secs(120)
        {
            let seconds = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(delay);
            if seconds > 60 {
                return Err(bad(
                    "Onshape is rate limiting requests. Please try again later.",
                ));
            }
            tokio::time::sleep(Duration::from_secs(seconds.max(1))).await;
            delay = (delay * 2).min(30);
            continue;
        }
        return Err(Error(
            if status.as_u16() == 401 {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_GATEWAY
            },
            match status.as_u16() {
                401 => "Onshape authorization expired. Sign in again.",
                402 => "Your Onshape annual API allowance has been reached.",
                403 => "Onshape denied access to this document or export.",
                404 => "Onshape document or tab was not found.",
                429 => "Onshape is rate limiting requests. Try again later.",
                _ => "Onshape could not complete the export. Try again.",
            }
            .into(),
        ));
    }
}
async fn json(response: reqwest::Response) -> Result<Value, Error> {
    let data = bytes(response, 2 * 1024 * 1024).await?;
    serde_json::from_slice(&data).map_err(|_| bad("Invalid response from Onshape"))
}
async fn bytes(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>, Error> {
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(bad("Onshape export is larger than the supported size"));
    }
    let mut out = vec![];
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| bad("Onshape download was interrupted"))?
    {
        if out.len() + chunk.len() > limit {
            return Err(bad("Onshape export is larger than the supported size"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}
fn step_data(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    if data.starts_with(b"PK\x03\x04") {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data))
            .map_err(|_| bad("Invalid Onshape export archive"))?;
        let mut selected = None;
        for i in 0..zip.len() {
            let f = zip
                .by_index(i)
                .map_err(|_| bad("Invalid Onshape export archive"))?;
            let name = f.name().to_lowercase();
            if name.ends_with(".step") || name.ends_with(".stp") {
                if selected.is_some() {
                    return Err(bad(
                        "Onshape returned multiple STEP files. Export a single combined assembly.",
                    ));
                }
                selected = Some(i);
            }
        }
        let mut f = zip
            .by_index(selected.ok_or_else(|| bad("No STEP file in Onshape export"))?)
            .map_err(|_| bad("Invalid export archive"))?;
        if f.size() > LIMIT as u64 {
            return Err(bad("Onshape export is too large"));
        }
        let mut out = vec![];
        std::io::Read::read_to_end(&mut std::io::Read::take(&mut f, LIMIT as u64 + 1), &mut out)
            .map_err(|_| bad("Could not read STEP export"))?;
        if out.len() > LIMIT {
            return Err(bad("Onshape export is too large"));
        }
        Ok(out)
    } else {
        Ok(data)
    }
}
async fn fetch(
    app: &App,
    w: &Shared,
    p: &Principal,
    s: &Source,
) -> Result<(Vec<u8>, String), Error> {
    let auth = &app.auth;
    let elements = json(
        request(
            auth,
            p,
            Method::GET,
            api_url(
                auth,
                &["documents", "d", &s.did, &s.kind, &s.wid, "elements"],
            ),
            None,
        )
        .await?,
    )
    .await?;
    let element = elements
        .as_array()
        .and_then(|v| v.iter().find(|e| e["id"].as_str() == Some(&s.eid)))
        .ok_or_else(|| bad("This Onshape tab was not found"))?;
    let resource = match element["elementType"].as_str() {
        Some("PARTSTUDIO") => "partstudios",
        Some("ASSEMBLY") => "assemblies",
        _ => return Err(bad("Select a Part Studio or assembly tab")),
    };
    let name = element["name"].as_str().unwrap_or("Onshape").to_string();
    w.lock().unwrap().status = format!("Exporting {name} from Onshape…");
    let mut body = json!({"storeInDocument":false,"formatName":"STEP","grouping":true});
    if let Some(configuration) = &s.configuration {
        body["configuration"] = json!(configuration);
    }
    let mut result = json(
        request(
            auth,
            p,
            Method::POST,
            api_url(
                auth,
                &[
                    resource,
                    "d",
                    &s.did,
                    &s.kind,
                    &s.wid,
                    "e",
                    &s.eid,
                    "translations",
                ],
            ),
            Some(&body),
        )
        .await?,
    )
    .await?;
    let id = result["id"]
        .as_str()
        .ok_or_else(|| bad("Onshape did not return an export ID"))?
        .to_string();
    let began = Instant::now();
    let mut delay = 2;
    loop {
        match result["requestState"].as_str() {
            Some("DONE") => break,
            Some("FAILED") => return Err(bad("Onshape could not translate this model to STEP")),
            _ => {}
        }
        if began.elapsed() > Duration::from_secs(600) {
            return Err(bad("Onshape export timed out. Try again later."));
        }
        tokio::time::sleep(Duration::from_secs(delay)).await;
        delay = (delay * 2).min(15);
        result = json(
            request(
                auth,
                p,
                Method::GET,
                api_url(auth, &["translations", &id]),
                None,
            )
            .await?,
        )
        .await?;
    }
    let ids = result["resultExternalDataIds"]
        .as_array()
        .ok_or_else(|| bad("Onshape returned no downloadable STEP data"))?;
    if ids.len() != 1 {
        return Err(bad(
            "Onshape returned multiple files. Select a single combined assembly.",
        ));
    }
    let fid = ids[0]
        .as_str()
        .ok_or_else(|| bad("Invalid Onshape export ID"))?;
    w.lock().unwrap().status = "Downloading STEP from Onshape…".into();
    let data = bytes(
        request(
            auth,
            p,
            Method::GET,
            api_url(auth, &["documents", "d", &s.did, "externaldata", fid]),
            None,
        )
        .await?,
        LIMIT,
    )
    .await?;
    Ok((step_data(data)?, name))
}
#[derive(Deserialize)]
pub struct Import {
    url: String,
}
fn begin(app: App, w: Shared, p: Principal, source: Source) -> Result<Json<Value>, Error> {
    {
        let mut w = w.lock().unwrap();
        idle(&w)?;
        w.busy = true;
        w.status = "Preparing Onshape export…".into();
    }
    let snap = w.lock().unwrap().snapshot();
    tokio::spawn(async move {
        let result = match fetch(&app, &w, &p, &source).await {
            Ok((data, name)) => {
                w.lock().unwrap().status = "Importing Onshape STEP…".into();
                let w2 = w.clone();
                blocking(move || {
                    import_bytes(
                        &app,
                        &w2,
                        &format!("{name}.step"),
                        &data,
                        Some(model::OnshapeSource {
                            url: source.url,
                            name,
                            imported_at: auth::now(),
                        }),
                    )
                })
                .await
            }
            Err(e) => Err(e),
        };
        let mut w = w.lock().unwrap();
        w.busy = false;
        if let Err(e) = result {
            w.status = e.1;
        } else {
            w.status = "Onshape parts imported".into();
        }
    });
    Ok(Json(snap))
}
pub async fn import(
    State(app): State<App>,
    Extension(p): Extension<Principal>,
    headers: HeaderMap,
    Json(input): Json<Import>,
) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    begin(app, w, p, parse(&input.url)?)
}
pub async fn refresh(
    State(app): State<App>,
    Extension(p): Extension<Principal>,
    headers: HeaderMap,
) -> Result<Json<Value>, Error> {
    let w = session(&app, &headers)?;
    let url = w
        .lock()
        .unwrap()
        .saved
        .onshape
        .as_ref()
        .map(|s| s.url.clone())
        .ok_or_else(|| bad("No Onshape source is linked to this workspace"))?;
    begin(app, w, p, parse(&url)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn links_are_pinned_to_onshape_and_preserve_configuration() {
        let url = "https://cad.onshape.com/documents/0123456789abcdef01234567/w/0123456789abcdef01234567/e/0123456789abcdef01234567?configuration=foo%3Dbar%3Bbaz%3Dtwo%20words#ignored";
        let s = parse(url).unwrap();
        assert_eq!(s.configuration.as_deref(), Some("foo=bar;baz=two words"));
        assert!(!s.url.contains('#'));
        for invalid in [
            url.replace("cad.onshape.com", "evil.example"),
            url.replace("https://", "http://"),
            url.replace("/w/", "/m/"),
            url.replace("cad.onshape.com", "cad.onshape.com:444"),
            url.replace("/documents/", "/documents/../"),
        ] {
            assert!(parse(&invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn archive_accepts_one_step_without_extracting_paths() {
        let data = zip_files(vec![(
            "../../assembly.step".into(),
            b"ISO-10303-21".to_vec(),
        )])
        .unwrap();
        assert_eq!(step_data(data).unwrap(), b"ISO-10303-21");
        assert!(
            step_data(
                zip_files(vec![("a.step".into(), vec![]), ("b.step".into(), vec![])]).unwrap()
            )
            .is_err()
        );
    }
}
