// webdav_service.rs - WebDAV sync for the 7 note files
use crate::credential_manager::{get_credential, store_credential, WEBDAV_SERVICE};
use chrono::{DateTime, Local, Utc};
use log::{error, info, warn};
use reqwest::{header, Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const NOTE_COUNT: usize = 7;
const CONFIG_FILE: &str = "webdav.json";
const STATE_FILE: &str = "webdav_state.json";

// Prevents two syncs from running at the same time
static SYNC_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct WebDavConfig {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub remote_dir: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct NoteState {
    hash: String,
    token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct SyncState {
    // url + remote dir the state belongs to; state is discarded when it changes
    target: String,
    notes: HashMap<usize, NoteState>,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub pulled: Vec<usize>,
    pub pushed: Vec<usize>,
    pub conflicts: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Skip,
    Push,
    Pull,
    Conflict,
}

/// Pure decision logic. `remote_token` is the ETag (or Last-Modified) of the
/// remote file, `None` if it does not exist.
fn decide(
    local_hash: &str,
    local_empty: bool,
    state: Option<&NoteState>,
    remote_token: Option<&str>,
) -> Decision {
    let Some(remote_token) = remote_token else {
        // Remote missing: upload if there is anything to upload
        return if local_empty {
            Decision::Skip
        } else {
            Decision::Push
        };
    };

    match state {
        None => {
            if local_empty {
                Decision::Pull
            } else {
                Decision::Conflict
            }
        }
        Some(state) => {
            let local_changed = state.hash != local_hash;
            let remote_changed = state.token.as_deref() != Some(remote_token);
            match (local_changed, remote_changed) {
                (false, false) => Decision::Skip,
                (true, false) => Decision::Push,
                (false, true) => Decision::Pull,
                (true, true) => Decision::Conflict,
            }
        }
    }
}

fn hash_content(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    Ok(dir)
}

fn load_config(app: &AppHandle) -> Result<WebDavConfig, String> {
    let path = data_dir(app)?.join(CONFIG_FILE);
    if !path.exists() {
        return Ok(WebDavConfig::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read WebDAV config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse WebDAV config: {}", e))
}

fn load_state(app: &AppHandle, target: &str) -> SyncState {
    let state = data_dir(app)
        .ok()
        .map(|d| d.join(STATE_FILE))
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|c| serde_json::from_str::<SyncState>(&c).ok());
    match state {
        Some(s) if s.target == target => s,
        _ => SyncState {
            target: target.to_string(),
            notes: HashMap::new(),
        },
    }
}

fn save_state(app: &AppHandle, state: &SyncState) -> Result<(), String> {
    let path = data_dir(app)?.join(STATE_FILE);
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize sync state: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed to save sync state: {}", e))
}

fn base_url(config: &WebDavConfig) -> String {
    config.url.trim().trim_end_matches('/').to_string()
}

fn dir_url(config: &WebDavConfig) -> String {
    let dir = config.remote_dir.trim().trim_matches('/');
    if dir.is_empty() {
        base_url(config)
    } else {
        format!("{}/{}", base_url(config), dir)
    }
}

fn target_id(config: &WebDavConfig) -> String {
    format!("{}|{}", dir_url(config), config.username)
}

fn file_url(config: &WebDavConfig, name: &str) -> String {
    format!("{}/{}", dir_url(config), name)
}

/// Formats a reqwest error including its full source chain (e.g. TLS
/// certificate errors behind a corporate proxy) and logs it.
fn connection_error(e: reqwest::Error) -> String {
    let mut msg = e.to_string();
    let mut source = std::error::Error::source(&e);
    while let Some(s) = source {
        msg.push_str(&format!(": {}", s));
        source = s.source();
    }
    log::error!("WebDAV connection error: {}", msg);
    format!("Connection failed: {}", msg)
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

fn authed(req: RequestBuilder, config: &WebDavConfig, password: &str) -> RequestBuilder {
    req.basic_auth(&config.username, Some(password))
}

fn check_status(status: StatusCode, what: &str) -> Result<(), String> {
    match status {
        s if s.is_success() => Ok(()),
        StatusCode::UNAUTHORIZED => {
            Err("Authentication failed (401): check user and password".into())
        }
        StatusCode::FORBIDDEN => Err("Access denied (403)".into()),
        s => Err(format!("{} failed with HTTP {}", what, s)),
    }
}

fn token_from(res: &Response) -> Option<String> {
    let headers = res.headers();
    headers
        .get(header::ETAG)
        .or_else(|| headers.get(header::LAST_MODIFIED))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn modified_from(res: &Response) -> Option<SystemTime> {
    res.headers()
        .get(header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| DateTime::parse_from_rfc2822(s).ok())
        .map(|dt| SystemTime::from(dt.with_timezone(&Utc)))
}

/// Creates the remote directory (MKCOL). An existing directory (405) is fine.
async fn ensure_remote_dir(
    client: &Client,
    config: &WebDavConfig,
    password: &str,
) -> Result<(), String> {
    if dir_url(config) == base_url(config) {
        return Ok(());
    }
    let method = Method::from_bytes(b"MKCOL").map_err(|e| e.to_string())?;
    let res = authed(client.request(method, dir_url(config)), config, password)
        .send()
        .await
        .map_err(connection_error)?;
    if res.status() == StatusCode::METHOD_NOT_ALLOWED {
        return Ok(());
    }
    check_status(res.status(), "Creating remote directory")
}

struct RemoteHead {
    token: String,
}

async fn head_remote(
    client: &Client,
    config: &WebDavConfig,
    password: &str,
    name: &str,
) -> Result<Option<RemoteHead>, String> {
    let res = authed(client.head(file_url(config, name)), config, password)
        .send()
        .await
        .map_err(connection_error)?;
    if res.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    check_status(res.status(), "Checking remote file")?;
    // Servers without ETag/Last-Modified: fall back to a constant so the file
    // is at least detected as existing.
    let token = token_from(&res).unwrap_or_default();
    Ok(Some(RemoteHead { token }))
}

struct RemoteFile {
    content: String,
    token: Option<String>,
    modified: Option<SystemTime>,
}

async fn get_remote(
    client: &Client,
    config: &WebDavConfig,
    password: &str,
    name: &str,
) -> Result<RemoteFile, String> {
    let res = authed(client.get(file_url(config, name)), config, password)
        .send()
        .await
        .map_err(connection_error)?;
    check_status(res.status(), "Downloading remote file")?;
    let token = token_from(&res);
    let modified = modified_from(&res);
    let content = res
        .text()
        .await
        .map_err(|e| format!("Failed to read remote file: {}", e))?;
    Ok(RemoteFile {
        content,
        token,
        modified,
    })
}

async fn put_remote(
    client: &Client,
    config: &WebDavConfig,
    password: &str,
    name: &str,
    content: &str,
) -> Result<(), String> {
    let res = authed(client.put(file_url(config, name)), config, password)
        .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
        .body(content.to_string())
        .send()
        .await
        .map_err(connection_error)?;
    check_status(res.status(), "Uploading file")
}

fn get_password(app: &AppHandle, config: &WebDavConfig) -> Result<String, String> {
    get_credential(app, WEBDAV_SERVICE, &config.username)
        .map_err(|_| "No WebDAV password stored".to_string())
}

fn read_local(path: &PathBuf) -> (String, Option<SystemTime>) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let modified = fs::metadata(path).and_then(|m| m.modified()).ok();
    (content, modified)
}

async fn sync_inner(
    app: &AppHandle,
    config: &WebDavConfig,
    password: &str,
) -> Result<SyncResult, String> {
    let client = build_client()?;
    ensure_remote_dir(&client, config, password).await?;

    let mut state = load_state(app, &target_id(config));
    let mut result = SyncResult::default();
    let mut backup_done = false;

    for i in 0..NOTE_COUNT {
        let name = format!("note_{}.md", i);
        let path = crate::storage_service::get_note_path(app, i);
        let (local, local_modified) = read_local(&path);
        let local_hash = hash_content(&local);
        let remote = head_remote(&client, config, password, &name).await?;

        let decision = decide(
            &local_hash,
            local.trim().is_empty(),
            state.notes.get(&i),
            remote.as_ref().map(|r| r.token.as_str()),
        );

        // Local overwrite helper: backup once per sync before touching local files
        macro_rules! backup_once {
            () => {
                if !backup_done {
                    if let Err(e) = crate::backup_service::create_backup(app.clone()).await {
                        warn!("Backup before WebDAV pull failed: {}", e);
                    }
                    backup_done = true;
                }
            };
        }

        match decision {
            Decision::Skip => {
                // Record baseline if missing (e.g. both empty)
                if remote.is_some() || !state.notes.contains_key(&i) {
                    state.notes.entry(i).or_insert(NoteState {
                        hash: local_hash,
                        token: remote.map(|r| r.token),
                    });
                }
            }
            Decision::Push => {
                put_remote(&client, config, password, &name, &local).await?;
                let token = head_remote(&client, config, password, &name)
                    .await?
                    .map(|r| r.token);
                state.notes.insert(
                    i,
                    NoteState {
                        hash: local_hash,
                        token,
                    },
                );
                result.pushed.push(i);
            }
            Decision::Pull => {
                let file = get_remote(&client, config, password, &name).await?;
                backup_once!();
                fs::write(&path, &file.content)
                    .map_err(|e| format!("Failed to write note {}: {}", i, e))?;
                state.notes.insert(
                    i,
                    NoteState {
                        hash: hash_content(&file.content),
                        token: file.token,
                    },
                );
                result.pulled.push(i);
            }
            Decision::Conflict => {
                let file = get_remote(&client, config, password, &name).await?;
                if hash_content(&file.content) == local_hash {
                    // Same content on both sides: just record the baseline
                    state.notes.insert(
                        i,
                        NoteState {
                            hash: local_hash,
                            token: file.token,
                        },
                    );
                    continue;
                }

                let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                let conflict_name = format!("note_{}.conflict-{}.md", i, stamp);
                // Newer version wins; if remote time is unknown, local wins
                let remote_wins = match (file.modified, local_modified) {
                    (Some(r), Some(l)) => r > l,
                    _ => false,
                };

                if remote_wins {
                    // Keep the losing (local) version on the server
                    put_remote(&client, config, password, &conflict_name, &local).await?;
                    backup_once!();
                    fs::write(&path, &file.content)
                        .map_err(|e| format!("Failed to write note {}: {}", i, e))?;
                    state.notes.insert(
                        i,
                        NoteState {
                            hash: hash_content(&file.content),
                            token: file.token,
                        },
                    );
                    result.pulled.push(i);
                } else {
                    // Keep the losing (remote) version, then upload local
                    put_remote(&client, config, password, &conflict_name, &file.content).await?;
                    put_remote(&client, config, password, &name, &local).await?;
                    let token = head_remote(&client, config, password, &name)
                        .await?
                        .map(|r| r.token);
                    state.notes.insert(
                        i,
                        NoteState {
                            hash: local_hash,
                            token,
                        },
                    );
                    result.pushed.push(i);
                }
                warn!(
                    "WebDAV sync conflict on note {}, saved as {}",
                    i, conflict_name
                );
                result.conflicts.push(i);
            }
        }
    }

    save_state(app, &state)?;
    Ok(result)
}

#[tauri::command]
pub fn get_webdav_config(app_handle: AppHandle) -> Result<WebDavConfig, String> {
    load_config(&app_handle)
}

#[tauri::command]
pub fn save_webdav_config(
    app_handle: AppHandle,
    config: WebDavConfig,
    password: Option<String>,
) -> Result<(), String> {
    if let Some(pw) = password.filter(|p| !p.is_empty()) {
        store_credential(&app_handle, WEBDAV_SERVICE, &config.username, &pw)?;
    }
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize WebDAV config: {}", e))?;
    fs::write(data_dir(&app_handle)?.join(CONFIG_FILE), json)
        .map_err(|e| format!("Failed to save WebDAV config: {}", e))?;
    info!("WebDAV config saved (enabled: {})", config.enabled);
    Ok(())
}

#[tauri::command]
pub async fn test_webdav_connection(
    app_handle: AppHandle,
    config: WebDavConfig,
    password: Option<String>,
) -> Result<(), String> {
    if config.url.trim().is_empty() {
        return Err("Server URL is empty".into());
    }
    let password = match password.filter(|p| !p.is_empty()) {
        Some(p) => p,
        None => get_password(&app_handle, &config)?,
    };
    let client = build_client()?;
    ensure_remote_dir(&client, &config, &password).await?;
    // PROPFIND on the target directory verifies it is reachable
    let method = Method::from_bytes(b"PROPFIND").map_err(|e| e.to_string())?;
    let res = authed(client.request(method, dir_url(&config)), &config, &password)
        .header("Depth", "0")
        .send()
        .await
        .map_err(connection_error)?;
    if res.status() == StatusCode::NOT_FOUND {
        return Err("Remote directory not found (404)".into());
    }
    check_status(res.status(), "Connection test")
}

#[tauri::command]
pub async fn sync_webdav(app_handle: AppHandle) -> Result<SyncResult, String> {
    let config = load_config(&app_handle)?;
    if !config.enabled {
        return Ok(SyncResult::default());
    }
    let password = get_password(&app_handle, &config)?;

    let _guard = SYNC_LOCK.lock().await;
    match sync_inner(&app_handle, &config, &password).await {
        Ok(result) => {
            info!(
                "WebDAV sync done: pulled {:?}, pushed {:?}, conflicts {:?}",
                result.pulled, result.pushed, result.conflicts
            );
            Ok(result)
        }
        Err(e) => {
            error!("WebDAV sync failed: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(content: &str, token: &str) -> NoteState {
        NoteState {
            hash: hash_content(content),
            token: Some(token.into()),
        }
    }

    #[test]
    fn unchanged_is_skipped() {
        let s = st("a", "t1");
        assert_eq!(
            decide(&hash_content("a"), false, Some(&s), Some("t1")),
            Decision::Skip
        );
    }

    #[test]
    fn local_change_pushes() {
        let s = st("a", "t1");
        assert_eq!(
            decide(&hash_content("b"), false, Some(&s), Some("t1")),
            Decision::Push
        );
    }

    #[test]
    fn remote_change_pulls() {
        let s = st("a", "t1");
        assert_eq!(
            decide(&hash_content("a"), false, Some(&s), Some("t2")),
            Decision::Pull
        );
    }

    #[test]
    fn both_changed_conflicts() {
        let s = st("a", "t1");
        assert_eq!(
            decide(&hash_content("b"), false, Some(&s), Some("t2")),
            Decision::Conflict
        );
    }

    #[test]
    fn first_sync_cases() {
        assert_eq!(
            decide(&hash_content(""), true, None, Some("t")),
            Decision::Pull
        );
        assert_eq!(
            decide(&hash_content("x"), false, None, Some("t")),
            Decision::Conflict
        );
        assert_eq!(
            decide(&hash_content("x"), false, None, None),
            Decision::Push
        );
        assert_eq!(decide(&hash_content(""), true, None, None), Decision::Skip);
    }

    #[test]
    fn remote_deleted_pushes_local() {
        let s = st("a", "t1");
        assert_eq!(
            decide(&hash_content("a"), false, Some(&s), None),
            Decision::Push
        );
    }
}
