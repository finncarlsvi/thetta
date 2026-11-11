use super::*;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use axum::{extract::Query, middleware::Next, response::Redirect};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_COOKIE: &str = "__Host-thetta_session";
const FLOW_COOKIE: &str = "__Host-thetta_oauth";
const MONTH: i64 = 30 * 86400;
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn random() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|v| {
            let (key, value) = v.trim().split_once('=')?;
            (key == name).then(|| value.to_string())
        })
}
fn set_cookie(name: &str, value: &str, age: i64) -> String {
    format!("{name}={value}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={age}")
}
pub fn unauthorized() -> Error {
    Error(
        StatusCode::UNAUTHORIZED,
        "Sign in with Onshape to continue".into(),
    )
}
fn internal(e: impl std::fmt::Display) -> Error {
    eprintln!("Authentication storage error: {e}");
    Error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to complete authentication".into(),
    )
}
#[derive(Clone)]
pub struct Principal {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub workspace: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}
use serde::Serialize;
pub struct Auth {
    db: Mutex<Connection>,
    cipher: Aes256Gcm,
    pub origin: String,
    pub client_id: String,
    secret: String,
    allowed: Vec<String>,
    pub http: reqwest::Client,
    pub api_root: String,
    pub oauth_root: String,
    pub refresh_lock: tokio::sync::Mutex<()>,
}
impl Auth {
    pub fn new(
        path: &std::path::Path,
        key: &[u8],
        origin: String,
        client_id: String,
        secret: String,
        allowed: Vec<String>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(key.len() == 32, "Token encryption key must be 32 bytes");
        let db = Connection::open(path)?;
        db.execute_batch("PRAGMA journal_mode=WAL;
          CREATE TABLE IF NOT EXISTS users(id TEXT PRIMARY KEY,email TEXT NOT NULL,name TEXT NOT NULL,tokens BLOB NOT NULL);
          CREATE TABLE IF NOT EXISTS sessions(id TEXT PRIMARY KEY,user_id TEXT NOT NULL,workspace TEXT NOT NULL,expires INTEGER NOT NULL);
          CREATE TABLE IF NOT EXISTS states(id TEXT PRIMARY KEY,expires INTEGER NOT NULL);
          CREATE TABLE IF NOT EXISTS ownership(workspace TEXT PRIMARY KEY,user_id TEXT NOT NULL);")?;
        Ok(Self {
            db: Mutex::new(db),
            cipher: Aes256Gcm::new_from_slice(key).unwrap(),
            origin: origin.trim_end_matches('/').into(),
            client_id,
            secret,
            allowed,
            http: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(60))
                .build()?,
            api_root: "https://cad.onshape.com".into(),
            oauth_root: "https://oauth.onshape.com".into(),
            refresh_lock: Default::default(),
        })
    }
    pub fn from_env(root: &std::path::Path) -> anyhow::Result<Self> {
        let key = std::fs::read(std::env::var("THETTA_TOKEN_KEY_FILE")?)?;
        let client_id = std::env::var("ONSHAPE_CLIENT_ID_FILE")
            .ok()
            .map(std::fs::read_to_string)
            .transpose()?
            .unwrap_or_default()
            .trim()
            .into();
        let secret = std::env::var("ONSHAPE_CLIENT_SECRET_FILE")
            .ok()
            .map(std::fs::read_to_string)
            .transpose()?
            .unwrap_or_default()
            .trim()
            .into();
        let allowed = std::env::var("THETTA_ALLOWED_EMAILS")
            .unwrap_or_else(|_| "finncarlsvi@gmail.com".into())
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect();
        let origin =
            std::env::var("THETTA_PUBLIC_URL").unwrap_or_else(|_| "https://thetta.finn.xyz".into());
        anyhow::ensure!(
            origin.starts_with("https://") && reqwest::Url::parse(&origin)?.path() == "/",
            "THETTA_PUBLIC_URL must be an HTTPS origin"
        );
        Self::new(
            &root.join("auth.sqlite"),
            &key,
            origin,
            client_id,
            secret,
            allowed,
        )
    }
    pub fn workspace_owners(&self) -> anyhow::Result<Vec<(String, String)>> {
        let db = self.db.lock().unwrap();
        let mut statement =
            db.prepare("SELECT workspace,user_id FROM ownership ORDER BY workspace")?;
        let rows = statement.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
    pub fn ready(&self) -> bool {
        !self.client_id.is_empty() && !self.secret.is_empty()
    }
    pub fn callback(&self) -> String {
        format!("{}/api/auth/onshape/callback", self.origin)
    }
    fn seal(&self, t: &Tokens) -> Result<Vec<u8>, Error> {
        let id = Uuid::new_v4();
        let nonce = &id.as_bytes()[..12];
        let mut data = nonce.to_vec();
        data.extend(
            self.cipher
                .encrypt(
                    Nonce::from_slice(nonce),
                    serde_json::to_vec(t).map_err(internal)?.as_ref(),
                )
                .map_err(|_| internal("Token encryption failed"))?,
        );
        Ok(data)
    }
    fn unseal(&self, data: &[u8]) -> Result<Tokens, Error> {
        if data.len() < 12 {
            return Err(internal("Invalid token record"));
        }
        let data = self
            .cipher
            .decrypt(Nonce::from_slice(&data[..12]), &data[12..])
            .map_err(|_| internal("Token decryption failed"))?;
        serde_json::from_slice(&data).map_err(internal)
    }
    pub fn principal(&self, headers: &HeaderMap) -> Result<Principal, Error> {
        let id = cookie(headers, SESSION_COOKIE).ok_or_else(unauthorized)?;
        let p=self.db.lock().unwrap().query_row("SELECT u.id,u.email,u.name,s.workspace FROM sessions s JOIN users u ON u.id=s.user_id JOIN ownership o ON o.workspace=s.workspace AND o.user_id=u.id WHERE s.id=? AND s.expires>?",params![hash(&id),now()],|r|Ok(Principal{user_id:r.get(0)?,email:r.get(1)?,name:r.get(2)?,workspace:r.get(3)?})).optional().map_err(internal)?.ok_or_else(unauthorized)?;
        if !self.allowed.contains(&p.email.to_lowercase()) {
            return Err(Error(
                StatusCode::FORBIDDEN,
                "This Onshape account is not approved for Thetta".into(),
            ));
        }
        Ok(p)
    }
    fn create_session(
        &self,
        user: &str,
        email: &str,
        name: &str,
        tokens: &Tokens,
        root: &std::path::Path,
    ) -> Result<String, Error> {
        if !self.allowed.contains(&email.to_lowercase()) {
            return Err(Error(
                StatusCode::FORBIDDEN,
                "This Onshape account is not approved for Thetta".into(),
            ));
        }
        let encrypted = self.seal(tokens)?;
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(internal)?;
        tx.execute("INSERT INTO users(id,email,name,tokens) VALUES(?,?,?,?) ON CONFLICT(id) DO UPDATE SET email=excluded.email,name=excluded.name,tokens=excluded.tokens",params![user,email,name,encrypted]).map_err(internal)?;
        // This installation was private and has one approved owner. Adopt legacy workspace UUIDs without changing their files.
        if self.allowed.len() == 1 {
            for entry in std::fs::read_dir(root).map_err(internal)? {
                let entry = entry.map_err(internal)?;
                let id = entry.file_name().to_string_lossy().into_owned();
                if Uuid::parse_str(&id).is_ok() && entry.path().join("workspace.json").is_file() {
                    tx.execute(
                        "INSERT OR IGNORE INTO ownership(workspace,user_id) VALUES(?,?)",
                        params![id, user],
                    )
                    .map_err(internal)?;
                }
            }
        }
        let workspace = tx
            .query_row(
                "SELECT workspace FROM ownership WHERE user_id=? ORDER BY workspace LIMIT 1",
                [user],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(internal)?
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        tx.execute(
            "INSERT OR IGNORE INTO ownership(workspace,user_id) VALUES(?,?)",
            params![workspace, user],
        )
        .map_err(internal)?;
        let session = random();
        tx.execute(
            "INSERT INTO sessions VALUES(?,?,?,?)",
            params![hash(&session), user, workspace, now() + MONTH],
        )
        .map_err(internal)?;
        tx.execute("DELETE FROM sessions WHERE expires<=?", [now()])
            .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(session)
    }
    pub fn tokens(&self, user: &str) -> Result<Tokens, Error> {
        let data: Vec<u8> = self
            .db
            .lock()
            .unwrap()
            .query_row("SELECT tokens FROM users WHERE id=?", [user], |r| r.get(0))
            .map_err(|_| unauthorized())?;
        self.unseal(&data)
    }
    pub fn end_sessions(&self, user: &str) -> Result<(), Error> {
        self.db
            .lock()
            .unwrap()
            .execute("DELETE FROM sessions WHERE user_id=?", [user])
            .map_err(internal)?;
        Ok(())
    }
    pub fn expire_access(&self, user: &str) -> Result<(), Error> {
        let mut tokens = self.tokens(user)?;
        tokens.expires_at = 0;
        self.db
            .lock()
            .unwrap()
            .execute(
                "UPDATE users SET tokens=? WHERE id=?",
                params![self.seal(&tokens)?, user],
            )
            .map_err(internal)?;
        Ok(())
    }
    pub async fn access_token(&self, p: &Principal) -> Result<String, Error> {
        let _lock = self.refresh_lock.lock().await;
        let tokens = self.tokens(&p.user_id)?;
        if tokens.expires_at > now() + 60 {
            return Ok(tokens.access_token);
        }
        let response = self
            .http
            .post(format!("{}/oauth/token", self.oauth_root))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", tokens.refresh_token.as_str()),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.secret.as_str()),
            ])
            .send()
            .await
            .map_err(|_| bad("Onshape is temporarily unavailable"))?;
        if !response.status().is_success() {
            if response.status().is_client_error()
                && response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS
            {
                self.db
                    .lock()
                    .unwrap()
                    .execute("DELETE FROM sessions WHERE user_id=?", [&p.user_id])
                    .map_err(internal)?;
                return Err(unauthorized());
            }
            return Err(bad("Onshape token refresh is temporarily unavailable"));
        }
        let data: Value = response
            .json()
            .await
            .map_err(|_| bad("Invalid response from Onshape"))?;
        let refreshed = Tokens {
            access_token: data["access_token"]
                .as_str()
                .ok_or_else(unauthorized)?
                .into(),
            refresh_token: data["refresh_token"]
                .as_str()
                .unwrap_or(&tokens.refresh_token)
                .into(),
            expires_at: now() + data["expires_in"].as_i64().unwrap_or(3600),
        };
        let sealed = self.seal(&refreshed)?;
        self.db
            .lock()
            .unwrap()
            .execute(
                "UPDATE users SET tokens=? WHERE id=?",
                params![sealed, p.user_id],
            )
            .map_err(internal)?;
        Ok(refreshed.access_token)
    }
    #[cfg(test)]
    pub fn test_login(&self, root: &std::path::Path, email: &str) -> String {
        let id = email.split('@').next().unwrap();
        let sid = self
            .create_session(
                id,
                email,
                "Test user",
                &Tokens {
                    access_token: "test-access".into(),
                    refresh_token: "test-refresh".into(),
                    expires_at: now() + 3600,
                },
                root,
            )
            .unwrap();
        format!("{SESSION_COOKIE}={sid}")
    }
}
pub async fn gate(
    State(app): State<App>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, Error> {
    let p = app.auth.principal(request.headers())?;
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::HEAD {
        let origin = request
            .headers()
            .get("origin")
            .and_then(|v| v.to_str().ok());
        if origin != Some(app.auth.origin.as_str()) {
            return Err(Error(
                StatusCode::FORBIDDEN,
                "Request origin is not allowed".into(),
            ));
        }
    }
    let mut request = request;
    request.extensions_mut().insert(p);
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    Ok(response)
}
pub async fn status(State(app): State<App>, headers: HeaderMap) -> Response {
    let body = match app.auth.principal(&headers) {
        Ok(p) => {
            json!({"authenticated":true,"configured":app.auth.ready(),"user":{"id":p.user_id,"email":p.email,"name":p.name}})
        }
        Err(_) => json!({"authenticated":false,"configured":app.auth.ready()}),
    };
    ([("cache-control", "no-store")], Json(body)).into_response()
}
pub async fn login(State(app): State<App>) -> Result<Response, Error> {
    let auth = &app.auth;
    if !auth.ready() {
        return Err(Error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Onshape OAuth registration is not configured yet".into(),
        ));
    }
    let state = random();
    auth.db
        .lock()
        .unwrap()
        .execute("DELETE FROM states WHERE expires<=?", [now()])
        .map_err(internal)?;
    auth.db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO states VALUES(?,?)",
            params![hash(&state), now() + 600],
        )
        .map_err(internal)?;
    let mut url = reqwest::Url::parse(&format!("{}/oauth/authorize", auth.oauth_root)).unwrap();
    url.query_pairs_mut().extend_pairs([
        ("response_type", "code"),
        ("client_id", &auth.client_id),
        ("redirect_uri", &auth.callback()),
        ("scope", "OAuth2Read OAuth2ReadPII"),
        ("state", &state),
    ]);
    let mut response = Redirect::to(url.as_str()).into_response();
    response.headers_mut().insert(
        "set-cookie",
        set_cookie(FLOW_COOKIE, &state, 600).parse().unwrap(),
    );
    Ok(response)
}
#[derive(Deserialize)]
pub struct Callback {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}
pub async fn callback(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<Callback>,
) -> Result<Response, Error> {
    let browser = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("text/html"));
    match complete_callback(app, headers, q).await {
        Err(error) if browser => {
            let reason = if error.1.contains("expired") {
                "expired"
            } else if error.0 == StatusCode::FORBIDDEN {
                "not-approved"
            } else {
                "failed"
            };
            let mut response = Redirect::to(&format!("/?signin={reason}")).into_response();
            response.headers_mut().insert(
                "set-cookie",
                set_cookie(FLOW_COOKIE, "", 0).parse().unwrap(),
            );
            response
                .headers_mut()
                .insert("cache-control", "no-store".parse().unwrap());
            response
                .headers_mut()
                .insert("referrer-policy", "no-referrer".parse().unwrap());
            Ok(response)
        }
        result => result,
    }
}
async fn complete_callback(app: App, headers: HeaderMap, q: Callback) -> Result<Response, Error> {
    let state = q.state.ok_or_else(|| bad("Missing OAuth state"))?;
    let bound = cookie(&headers, FLOW_COOKIE)
        .ok_or_else(|| bad("OAuth session expired. Sign in again."))?;
    if hash(&state) != hash(&bound) {
        return Err(bad("Invalid OAuth state"));
    }
    let used = app
        .auth
        .db
        .lock()
        .unwrap()
        .execute(
            "DELETE FROM states WHERE id=? AND expires>?",
            params![hash(&state), now()],
        )
        .map_err(internal)?;
    if used != 1 {
        return Err(bad("OAuth state expired or already used"));
    }
    if q.error.is_some() {
        return Ok(Redirect::to("/?signin=denied").into_response());
    }
    let code = q.code.ok_or_else(|| bad("Missing authorization code"))?;
    let auth = &app.auth;
    let response = auth
        .http
        .post(format!("{}/oauth/token", auth.oauth_root))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", auth.client_id.as_str()),
            ("client_secret", auth.secret.as_str()),
            ("redirect_uri", auth.callback().as_str()),
        ])
        .send()
        .await
        .map_err(|_| bad("Onshape is temporarily unavailable"))?;
    if !response.status().is_success() {
        return Err(bad(
            "Onshape rejected the authorization code. Sign in again.",
        ));
    }
    let t: Value = response
        .json()
        .await
        .map_err(|_| bad("Invalid token response"))?;
    let tokens = Tokens {
        access_token: t["access_token"].as_str().ok_or_else(unauthorized)?.into(),
        refresh_token: t["refresh_token"].as_str().ok_or_else(unauthorized)?.into(),
        expires_at: now() + t["expires_in"].as_i64().unwrap_or(3600),
    };
    let response = auth
        .http
        .get(format!("{}/api/users/sessioninfo", auth.api_root))
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .map_err(|_| bad("Could not verify Onshape identity"))?;
    if !response.status().is_success() {
        return Err(bad("Could not verify Onshape identity"));
    }
    let profile: Value = response
        .json()
        .await
        .map_err(|_| bad("Invalid profile response"))?;
    let id = profile["id"]
        .as_str()
        .ok_or_else(|| bad("Onshape did not return a user ID"))?;
    let email = profile["email"]
        .as_str()
        .ok_or_else(|| bad("Onshape did not return an email address"))?;
    let name = profile["name"].as_str().unwrap_or(email);
    let auth = auth.clone();
    let root = app.root.clone();
    let id = id.to_owned();
    let email = email.to_owned();
    let name = name.to_owned();
    let sid = blocking(move || auth.create_session(&id, &email, &name, &tokens, &root)).await?;
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().append(
        "set-cookie",
        set_cookie(SESSION_COOKIE, &sid, MONTH).parse().unwrap(),
    );
    response.headers_mut().append(
        "set-cookie",
        set_cookie(FLOW_COOKIE, "", 0).parse().unwrap(),
    );
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("referrer-policy", "no-referrer".parse().unwrap());
    Ok(response)
}
pub async fn logout(State(app): State<App>, headers: HeaderMap) -> Result<Response, Error> {
    if let Some(id) = cookie(&headers, SESSION_COOKIE) {
        app.auth
            .db
            .lock()
            .unwrap()
            .execute("DELETE FROM sessions WHERE id=?", [hash(&id)])
            .map_err(internal)?;
    }
    Ok((
        [("set-cookie", set_cookie(SESSION_COOKIE, "", 0))],
        Json(json!({"ok":true})),
    )
        .into_response())
}
pub async fn workspaces(State(app): State<App>, headers: HeaderMap) -> Result<Response, Error> {
    let p = app.auth.principal(&headers)?;
    let rows: Vec<String> = {
        let db = app.auth.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT workspace FROM ownership WHERE user_id=? ORDER BY workspace")
            .map_err(internal)?;
        stmt.query_map([p.user_id], |r| r.get(0))
            .map_err(internal)?
            .collect::<Result<_, _>>()
            .map_err(internal)?
    };
    let mut items = vec![];
    for id in rows {
        let file = app.root.join(&id).join("workspace.json");
        let updated_at = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        let saved: Saved = std::fs::read(&file)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        // Small contact-sheet previews keep the explorer independent of native model loading.
        let preview: Vec<_> = saved
            .parts
            .iter()
            .take(6)
            .map(|part| {
                let polygon = &part.piece.polygon;
                let outer: Vec<_> = polygon
                    .outer
                    .iter()
                    .step_by(polygon.outer.len().div_ceil(64).max(1))
                    .collect();
                let holes: Vec<_> = polygon
                    .holes
                    .iter()
                    .take(4)
                    .map(|ring| {
                        ring.iter()
                            .step_by(ring.len().div_ceil(64).max(1))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                json!({"outer":outer,"holes":holes})
            })
            .collect();
        let source = if saved.onshape.is_some() {
            "onshape"
        } else if saved.filename.is_some() {
            "step"
        } else {
            "empty"
        };
        items.push(json!({"id":id,"filename":saved.filename,"parts":saved.parts.len(),"active":p.workspace==id,"updated_at":updated_at,"source":source,"preview":preview}));
    }
    Ok(([("cache-control", "no-store")], Json(json!(items))).into_response())
}
#[derive(Deserialize)]
pub struct Selection {
    id: Option<String>,
}
pub async fn select_workspace(
    State(app): State<App>,
    headers: HeaderMap,
    Json(selection): Json<Selection>,
) -> Result<Json<Value>, Error> {
    let p = app.auth.principal(&headers)?;
    let id = selection.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    if Uuid::parse_str(&id).is_err() {
        return Err(bad("Invalid workspace ID"));
    }
    let mut db = app.auth.db.lock().unwrap();
    let tx = db.transaction().map_err(internal)?;
    let owner: Option<String> = tx
        .query_row(
            "SELECT user_id FROM ownership WHERE workspace=?",
            [&id],
            |r| r.get(0),
        )
        .optional()
        .map_err(internal)?;
    if owner.as_ref().is_some_and(|u| u != &p.user_id)
        || (owner.is_none() && app.root.join(&id).exists())
    {
        return Err(Error(
            StatusCode::FORBIDDEN,
            "Workspace access denied".into(),
        ));
    }
    tx.execute(
        "INSERT OR IGNORE INTO ownership VALUES(?,?)",
        params![id, p.user_id],
    )
    .map_err(internal)?;
    let sid = cookie(&headers, SESSION_COOKIE).ok_or_else(unauthorized)?;
    tx.execute(
        "UPDATE sessions SET workspace=? WHERE id=?",
        params![id, hash(&sid)],
    )
    .map_err(internal)?;
    tx.commit().map_err(internal)?;
    Ok(Json(json!({"ok":true,"id":id})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    fn app(root: &std::path::Path) -> App {
        App {
            root: root.into(),
            workspaces: Default::default(),
            occt: Default::default(),
            shutting_down: Default::default(),
            auth: Arc::new(
                Auth::new(
                    &root.join("auth.sqlite"),
                    &[7; 32],
                    "https://thetta.finn.xyz".into(),
                    "client".into(),
                    "secret".into(),
                    vec!["owner@example.com".into(), "other@example.com".into()],
                )
                .unwrap(),
            ),
        }
    }
    fn headers(cookie: &str) -> HeaderMap {
        HeaderMap::from_iter([("cookie".parse().unwrap(), cookie.parse().unwrap())])
    }
    async fn call(
        app: &App,
        method: &str,
        path: &str,
        cookie: &str,
        origin: Option<&str>,
        body: Option<Value>,
    ) -> Response {
        let mut r = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", cookie);
        if let Some(o) = origin {
            r = r.header("origin", o);
        }
        if body.is_some() {
            r = r.header("content-type", "application/json");
        }
        super::super::router(app.clone(), PathBuf::from("web/dist"))
            .oneshot(
                r.body(
                    body.map(|v| Body::from(v.to_string()))
                        .unwrap_or(Body::empty()),
                )
                .unwrap(),
            )
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn all_data_routes_require_login_and_mutations_require_origin() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        for (method, path) in [
            ("POST", "/api/workspace"),
            ("GET", "/api/workspaces"),
            ("GET", "/api/events"),
            ("POST", "/api/upload"),
            ("PUT", "/api/settings"),
            ("PUT", "/api/parts/0"),
            ("PUT", "/api/thickness"),
            ("POST", "/api/nesting/start"),
            ("POST", "/api/nesting/improve"),
            ("POST", "/api/nesting/stop"),
            ("POST", "/api/export/step"),
            ("POST", "/api/export/gcode"),
            ("POST", "/api/onshape/import"),
            ("POST", "/api/onshape/refresh"),
            ("POST", "/api/tabs"),
        ] {
            assert_eq!(
                call(&app, method, path, "", None, None).await.status(),
                StatusCode::UNAUTHORIZED,
                "{path}"
            );
        }
        let sid = app.auth.test_login(root.path(), "owner@example.com");
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/workspace",
                &sid,
                Some("https://evil.example"),
                None
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            call(&app, "POST", "/api/workspace", &sid, None, None)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/workspace",
                &sid,
                Some(&app.auth.origin),
                None
            )
            .await
            .status(),
            StatusCode::OK
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/auth/logout",
                &sid,
                Some(&app.auth.origin),
                None
            )
            .await
            .status(),
            StatusCode::OK
        );
        assert!(app.auth.principal(&headers(&sid)).is_err());
    }
    #[tokio::test]
    async fn explorer_lists_only_owned_projects_and_switches_without_rewriting_them() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        let sid = app.auth.test_login(root.path(), "owner@example.com");
        let owner = app.auth.principal(&headers(&sid)).unwrap();
        let other_sid = app.auth.test_login(root.path(), "other@example.com");
        let points: Vec<_> = (0..200).map(|i| json!({"x":i,"y":i % 7})).collect();
        let mut saved = Saved::default();
        saved.filename = Some("Wood storage.step".into());
        saved.onshape = Some(model::OnshapeSource {
            url: "https://cad.onshape.com/documents/example".into(),
            name: "Wood storage".into(),
            imported_at: now(),
        });
        saved.parts = (0..10).map(|id| serde_json::from_value(json!({"piece":{"id":id,"quantity":1,"polygon":{"outer":points,"holes":[points,points,points,points,points]}},"enabled":true,"rotation":"continuous"})).unwrap()).collect();
        let dir = root.path().join(&owner.workspace);
        std::fs::create_dir_all(&dir).unwrap();
        let original = serde_json::to_vec(&saved).unwrap();
        std::fs::write(dir.join("workspace.json"), &original).unwrap();
        let response = call(&app, "GET", "/api/workspaces", &sid, None, None).await;
        assert_eq!(response.headers()["cache-control"], "no-store");
        let projects: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(projects.as_array().unwrap().len(), 1);
        assert_eq!(projects[0]["filename"], "Wood storage.step");
        assert_eq!(projects[0]["parts"], 10);
        assert_eq!(projects[0]["source"], "onshape");
        assert!(projects[0]["updated_at"].is_u64());
        assert_eq!(projects[0]["preview"].as_array().unwrap().len(), 6);
        assert!(projects[0]["preview"][0]["outer"].as_array().unwrap().len() <= 64);
        assert_eq!(
            projects[0]["preview"][0]["holes"].as_array().unwrap().len(),
            4
        );
        let response = call(&app, "GET", "/api/workspaces", &other_sid, None, None).await;
        let projects: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(
            !projects
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["id"] == owner.workspace)
        );
        let response = call(
            &app,
            "POST",
            "/api/workspaces",
            &sid,
            Some(&app.auth.origin),
            Some(json!({"id":null})),
        )
        .await;
        let created: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        let new_id = created["id"].as_str().unwrap();
        assert_ne!(new_id, owner.workspace);
        assert_eq!(
            app.auth.principal(&headers(&sid)).unwrap().workspace,
            new_id
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/workspaces",
                &sid,
                Some(&app.auth.origin),
                Some(json!({"id":owner.workspace}))
            )
            .await
            .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.auth.principal(&headers(&sid)).unwrap().workspace,
            owner.workspace
        );
        assert_eq!(std::fs::read(dir.join("workspace.json")).unwrap(), original);
        assert!(app.workspaces.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn ownership_expiry_allowlist_and_encrypted_persistence() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        let sid = app.auth.test_login(root.path(), "owner@example.com");
        let owner = app.auth.principal(&headers(&sid)).unwrap();
        let other_sid = app.auth.test_login(root.path(), "other@example.com");
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/workspaces",
                &other_sid,
                Some(&app.auth.origin),
                Some(json!({"id":owner.workspace}))
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
        let token = Tokens {
            access_token: "SECRET-ACCESS".into(),
            refresh_token: "SECRET-REFRESH".into(),
            expires_at: now() + 3600,
        };
        assert!(
            app.auth
                .create_session("outsider", "bad@example.com", "No", &token, root.path())
                .is_err()
        );
        let encrypted = app.auth.seal(&token).unwrap();
        assert!(!String::from_utf8_lossy(&encrypted).contains("SECRET"));
        assert_eq!(
            app.auth.unseal(&encrypted).unwrap().refresh_token,
            "SECRET-REFRESH"
        );
        let restarted = self::app(root.path());
        assert_eq!(
            restarted.auth.principal(&headers(&sid)).unwrap().workspace,
            owner.workspace
        );
        app.auth
            .db
            .lock()
            .unwrap()
            .execute("UPDATE sessions SET expires=0", [])
            .unwrap();
        assert!(app.auth.principal(&headers(&sid)).is_err());
    }
    #[tokio::test]
    async fn oauth_state_is_bound_expiring_and_single_use() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        let response = call(&app, "GET", "/api/auth/onshape/start", "", None, None).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let location = Url::parse(response.headers()["location"].to_str().unwrap()).unwrap();
        let state = location
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .to_string();
        assert_eq!(
            call(
                &app,
                "GET",
                &format!("/api/auth/onshape/callback?state={state}&error=access_denied"),
                "",
                None,
                None
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        let path = format!("/api/auth/onshape/callback?state={state}&error=access_denied");
        assert_eq!(
            call(&app, "GET", &path, &cookie, None, None).await.status(),
            StatusCode::SEE_OTHER
        );
        assert_eq!(
            call(&app, "GET", &path, &cookie, None, None).await.status(),
            StatusCode::BAD_REQUEST
        );
        let response = call(&app, "GET", "/api/auth/onshape/start", "", None, None).await;
        let cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let state = cookie.split_once('=').unwrap().1;
        app.auth
            .db
            .lock()
            .unwrap()
            .execute("UPDATE states SET expires=0", [])
            .unwrap();
        assert_eq!(
            call(
                &app,
                "GET",
                &format!("/api/auth/onshape/callback?state={state}&error=denied"),
                &cookie,
                None,
                None
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        // Browsers recover on the login page, without retaining the provider code in the URL.
        for bound_cookie in ["", cookie.as_str()] {
            let response = super::super::router(app.clone(), PathBuf::from("web/dist"))
                .oneshot(
                    Request::builder()
                        .uri(format!(
                            "/api/auth/onshape/callback?state={state}&code=unused-code"
                        ))
                        .header("accept", "text/html,application/xhtml+xml")
                        .header("cookie", bound_cookie)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::SEE_OTHER);
            assert_eq!(response.headers()["location"], "/?signin=expired");
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["referrer-policy"], "no-referrer");
            assert!(
                response.headers()["set-cookie"]
                    .to_str()
                    .unwrap()
                    .contains("Max-Age=0")
            );
        }
    }
    use reqwest::Url;
    #[tokio::test]
    async fn real_oauth_exchange_uses_provider_identity_and_rotates_refresh_tokens() {
        let root = tempfile::tempdir().unwrap();
        let mut app = app(root.path());
        async fn token(
            axum::extract::Form(form): axum::extract::Form<HashMap<String, String>>,
        ) -> Json<Value> {
            assert_eq!(form["client_secret"], "secret");
            if form["grant_type"] == "authorization_code" {
                assert_eq!(form["code"], "valid-code");
                assert_eq!(
                    form["redirect_uri"],
                    "https://thetta.finn.xyz/api/auth/onshape/callback"
                );
            } else {
                assert_eq!(form["refresh_token"], "initial-refresh");
            }
            Json(
                json!({"access_token":"provider-access","refresh_token":if form["grant_type"]=="refresh_token"{"rotated-refresh"}else{"initial-refresh"},"expires_in":3600}),
            )
        }
        let provider = Router::new().route("/oauth/token", post(token)).route(
            "/api/users/sessioninfo",
            get(|h: HeaderMap| async move {
                assert_eq!(h["authorization"], "Bearer provider-access");
                Json(json!({"id":"provider-user-id","email":"owner@example.com","name":"Owner"}))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move { axum::serve(listener, provider).await.unwrap() });
        Arc::get_mut(&mut app.auth).unwrap().api_root = origin.clone();
        Arc::get_mut(&mut app.auth).unwrap().oauth_root = origin;
        let response = call(&app, "GET", "/api/auth/onshape/start", "", None, None).await;
        let cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let state = cookie.split_once('=').unwrap().1;
        let response = call(
            &app,
            "GET",
            &format!("/api/auth/onshape/callback?state={state}&code=valid-code"),
            &cookie,
            None,
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let session = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap())
            .find(|v| v.starts_with(SESSION_COOKIE))
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        let principal = app.auth.principal(&headers(session)).unwrap();
        assert_eq!(principal.user_id, "provider-user-id");
        let expired = app
            .auth
            .seal(&Tokens {
                access_token: "expired".into(),
                refresh_token: "initial-refresh".into(),
                expires_at: 0,
            })
            .unwrap();
        app.auth
            .db
            .lock()
            .unwrap()
            .execute("UPDATE users SET tokens=?", [expired])
            .unwrap();
        assert_eq!(
            app.auth.access_token(&principal).await.unwrap(),
            "provider-access"
        );
        assert_eq!(
            app.auth.tokens(&principal.user_id).unwrap().refresh_token,
            "rotated-refresh"
        );
        let response = call(&app, "GET", "/api/auth/status", session, None, None).await;
        let data = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(!String::from_utf8_lossy(&data).contains("provider-access"));
        handle.abort();
    }
}
