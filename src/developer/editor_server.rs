use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::json;

use crate::core::engine::asset_catalog::AssetCatalog;
use crate::data::enemy::EnemyDefinition;
use crate::data::relic::RelicDefinition;
use crate::data::world::level::LevelData;
use crate::data::world::prefab::LevelPrefabData;

const DEFAULT_EDITOR_PORT: u16 = 4327;
const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
const EDITOR_TOKEN_HEADER: &str = "x-cenotaph-editor-token";

#[derive(Debug, Serialize)]
struct LevelSummary {
    id: String,
    name: String,
    path: String,
    props: usize,
    modified_unix: u64,
}

#[derive(Debug, Serialize)]
struct PrefabSummary {
    id: String,
    name: String,
    path: String,
    props: usize,
    modified_unix: u64,
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

struct EditorSession {
    port: u16,
    token: String,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (listener, port) = bind_editor_listener()?;
    let token = generate_editor_token()?;
    let session = EditorSession { port, token };
    let url = format!("http://127.0.0.1:{}/?token={}", port, session.token);
    println!("[EDITOR] Cenotaph Level Editor is running.");
    println!("[EDITOR] Open {}", url);
    println!("[EDITOR] This URL includes a per-launch write token; do not share it.");
    println!("[EDITOR] Press Ctrl+C in this terminal to stop the editor server.");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream, &session) {
                    eprintln!("[EDITOR] Request failed: {}", error);
                }
            }
            Err(error) => eprintln!("[EDITOR] Connection failed: {}", error),
        }
    }

    Ok(())
}

fn bind_editor_listener() -> Result<(TcpListener, u16), Box<dyn std::error::Error>> {
    for port in DEFAULT_EDITOR_PORT..DEFAULT_EDITOR_PORT + 20 {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => return Ok((listener, port)),
            Err(_) => continue,
        }
    }

    Err(format!(
        "could not bind level editor server on ports {}-{}",
        DEFAULT_EDITOR_PORT,
        DEFAULT_EDITOR_PORT + 19
    )
    .into())
}

fn generate_editor_token() -> Result<String, Box<dyn std::error::Error>> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| {
        std::io::Error::other(format!(
            "failed to create editor session token: {:?}",
            error
        ))
    })?;
    Ok(bytes.iter().map(|byte| format!("{:02x}", byte)).collect())
}

fn handle_connection(stream: &mut TcpStream, session: &EditorSession) -> Result<(), String> {
    let request = read_http_request(stream)?;
    let response = if request_allowed(&request, session) {
        route_request(&request, session)
    } else {
        json_response(
            403,
            &json!({
                "ok": false,
                "error": "editor request rejected by local security checks"
            }),
        )
    };
    stream
        .write_all(&response)
        .map_err(|error| format!("failed to write response: {}", error))
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut buffer = [0_u8; 8192];
    let mut bytes = Vec::new();
    let mut content_length = 0_usize;

    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| format!("failed to read request: {}", error))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);

        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("request body is too large".to_string());
        }

        if let Some(header_end) = find_header_end(&bytes) {
            let headers = String::from_utf8_lossy(&bytes[..header_end]);
            content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.trim().eq_ignore_ascii_case("content-length") {
                        value.trim().parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            let total = header_end + 4 + content_length;
            while bytes.len() < total {
                let read = stream
                    .read(&mut buffer)
                    .map_err(|error| format!("failed to read request body: {}", error))?;
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..read]);
                if bytes.len() > MAX_REQUEST_BYTES {
                    return Err("request body is too large".to_string());
                }
            }
            break;
        }
    }

    let header_end = find_header_end(&bytes).ok_or_else(|| "malformed HTTP request".to_string())?;
    let header_text = String::from_utf8_lossy(&bytes[..header_end]);
    let headers = parse_headers(&header_text);
    let request_line = header_text
        .lines()
        .next()
        .ok_or_else(|| "missing HTTP request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "missing HTTP method".to_string())?
        .to_string();
    let path = request_parts
        .next()
        .ok_or_else(|| "missing HTTP path".to_string())?
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let body_start = header_end + 4;
    let body_end = body_start.saturating_add(content_length).min(bytes.len());
    let body = String::from_utf8(bytes[body_start..body_end].to_vec())
        .map_err(|error| format!("request body is not UTF-8: {}", error))?;

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn parse_headers(header_text: &str) -> Vec<(String, String)> {
    header_text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect()
}

impl HttpRequest {
    fn header(&self, name: &str) -> Option<&str> {
        let normalized = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| key == &normalized)
            .map(|(_, value)| value.as_str())
    }
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn request_allowed(request: &HttpRequest, session: &EditorSession) -> bool {
    if !host_allowed(request.header("host"), session.port) {
        return false;
    }

    if !origin_allowed(request.header("origin"), session.port) {
        return false;
    }

    if request.path.starts_with("/api/") {
        return request
            .header(EDITOR_TOKEN_HEADER)
            .is_some_and(|token| token_matches(token, &session.token));
    }

    true
}

fn host_allowed(host: Option<&str>, port: u16) -> bool {
    let Some(host) = host else {
        return false;
    };
    let normalized = host.trim().to_ascii_lowercase();
    normalized == format!("127.0.0.1:{}", port) || normalized == format!("localhost:{}", port)
}

fn origin_allowed(origin: Option<&str>, port: u16) -> bool {
    let Some(origin) = origin else {
        return true;
    };
    let normalized = origin.trim().to_ascii_lowercase();
    normalized == format!("http://127.0.0.1:{}", port)
        || normalized == format!("http://localhost:{}", port)
}

fn token_matches(candidate: &str, expected: &str) -> bool {
    let candidate = candidate.as_bytes();
    let expected = expected.as_bytes();
    if candidate.len() != expected.len() {
        return false;
    }

    candidate
        .iter()
        .zip(expected.iter())
        .fold(0_u8, |diff, (left, right)| diff | (left ^ right))
        == 0
}

fn route_request(request: &HttpRequest, _session: &EditorSession) -> Vec<u8> {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => static_response(
            "text/html; charset=utf-8",
            include_str!("../../tools/level_editor/index.html"),
        ),
        ("GET", "/styles.css") => static_response(
            "text/css; charset=utf-8",
            include_str!("../../tools/level_editor/styles.css"),
        ),
        ("GET", "/app.js") => static_response(
            "application/javascript; charset=utf-8",
            include_str!("../../tools/level_editor/app.js"),
        ),
        ("GET", "/prefab-tools.js") => static_response(
            "application/javascript; charset=utf-8",
            include_str!("../../tools/level_editor/prefab-tools.js"),
        ),
        ("GET", "/api/project") => json_response(200, &project_payload()),
        ("POST", "/api/validate") => validate_level_response(&request.body),
        _ => {
            if request.method == "GET" {
                if let Some(level_id) = request.path.strip_prefix("/api/levels/") {
                    return read_level_response(level_id);
                }
            }
            if request.method == "PUT" {
                if let Some(level_id) = request.path.strip_prefix("/api/levels/") {
                    return save_level_response(level_id, &request.body);
                }
            }
            if request.method == "GET" {
                if let Some(prefab_id) = request.path.strip_prefix("/api/prefabs/") {
                    return read_prefab_response(prefab_id);
                }
            }
            if request.method == "PUT" {
                if let Some(prefab_id) = request.path.strip_prefix("/api/prefabs/") {
                    return save_prefab_response(prefab_id, &request.body);
                }
            }
            if request.method == "DELETE" {
                if let Some(prefab_id) = request.path.strip_prefix("/api/prefabs/") {
                    return delete_prefab_response(prefab_id);
                }
            }

            json_response(
                404,
                &json!({
                    "ok": false,
                    "error": format!("unknown editor route {} {}", request.method, request.path)
                }),
            )
        }
    }
}

fn project_payload() -> serde_json::Value {
    let asset_catalog = AssetCatalog::scan_project();
    let runtime_assets = AssetCatalog::scan("assets").runtime_models();
    json!({
        "ok": true,
        "levels": level_summaries(),
        "prefabs": prefab_summaries(),
        "assets": runtime_assets,
        "asset_catalog": asset_catalog,
        "enemies": load_enemy_definitions(),
        "relics": load_relic_definitions(),
    })
}

fn level_summaries() -> Vec<LevelSummary> {
    let mut levels = Vec::new();
    let Ok(entries) = fs::read_dir("levels") else {
        return levels;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !has_extension(&path, "json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let path_label = path.to_string_lossy().replace('\\', "/");
        let (name, props) = LevelData::try_load(&path_label)
            .map(|level| (level.name, level.props.len()))
            .unwrap_or_else(|_| (id.to_string(), 0));
        let modified_unix = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        levels.push(LevelSummary {
            id: id.to_string(),
            name,
            path: path_label,
            props,
            modified_unix,
        });
    }

    levels.sort_by(|left, right| left.id.cmp(&right.id));
    levels
}

fn prefab_summaries() -> Vec<PrefabSummary> {
    let mut prefabs = Vec::new();
    let Ok(entries) = fs::read_dir("prefabs") else {
        return prefabs;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !has_extension(&path, "json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let path_label = path.to_string_lossy().replace('\\', "/");
        let (name, props) = LevelPrefabData::try_load(&path)
            .map(|prefab| (prefab.name, prefab.props.len()))
            .unwrap_or_else(|_| (id.to_string(), 0));
        let modified_unix = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        prefabs.push(PrefabSummary {
            id: id.to_string(),
            name,
            path: path_label,
            props,
            modified_unix,
        });
    }

    prefabs.sort_by(|left, right| left.id.cmp(&right.id));
    prefabs
}

fn read_level_response(level_id: &str) -> Vec<u8> {
    let Ok(path) = level_path_for_id(level_id) else {
        return json_response(400, &json!({"ok": false, "error": "invalid level id"}));
    };
    let path_label = path.to_string_lossy().replace('\\', "/");
    match fs::read_to_string(&path) {
        Ok(source) => match serde_json::from_str::<serde_json::Value>(&source) {
            Ok(json_value) => json_response(
                200,
                &json!({
                    "ok": true,
                    "id": level_id.trim_end_matches(".json"),
                    "path": path_label,
                    "level": json_value,
                    "source": source,
                }),
            ),
            Err(error) => json_response(
                500,
                &json!({"ok": false, "error": format!("level JSON could not be parsed: {}", error)}),
            ),
        },
        Err(error) => json_response(
            404,
            &json!({"ok": false, "error": format!("failed to read level: {}", error)}),
        ),
    }
}

fn validate_level_response(body: &str) -> Vec<u8> {
    match parse_level_body(body) {
        Ok(level) => match level.validate() {
            Ok(()) => json_response(200, &json!({"ok": true, "errors": []})),
            Err(errors) => json_response(200, &json!({"ok": false, "errors": errors})),
        },
        Err(error) => json_response(400, &json!({"ok": false, "errors": [error]})),
    }
}

fn save_level_response(level_id: &str, body: &str) -> Vec<u8> {
    let Ok(path) = level_path_for_id(level_id) else {
        return json_response(400, &json!({"ok": false, "error": "invalid level id"}));
    };
    let level = match parse_level_body(body) {
        Ok(level) => level,
        Err(error) => return json_response(400, &json!({"ok": false, "errors": [error]})),
    };

    match level.validate() {
        Ok(()) => {}
        Err(errors) => return json_response(422, &json!({"ok": false, "errors": errors})),
    }

    let backup = match backup_level_file(&path) {
        Ok(backup) => backup,
        Err(error) => return json_response(500, &json!({"ok": false, "error": error})),
    };

    match level.save_to_path(&path) {
        Ok(()) => json_response(
            200,
            &json!({
                "ok": true,
                "id": level_id.trim_end_matches(".json"),
                "path": path.to_string_lossy().replace('\\', "/"),
                "backup": backup.map(|path| path.to_string_lossy().replace('\\', "/")),
            }),
        ),
        Err(error) => json_response(500, &json!({"ok": false, "error": error})),
    }
}

fn read_prefab_response(prefab_id: &str) -> Vec<u8> {
    let Ok(path) = prefab_path_for_id(prefab_id) else {
        return json_response(400, &json!({"ok": false, "error": "invalid prefab id"}));
    };
    match LevelPrefabData::try_load(&path) {
        Ok(prefab) => json_response(
            200,
            &json!({
                "ok": true,
                "id": prefab_id.trim_end_matches(".json"),
                "path": path.to_string_lossy().replace('\\', "/"),
                "prefab": prefab,
            }),
        ),
        Err(error) => json_response(404, &json!({"ok": false, "error": error})),
    }
}

fn save_prefab_response(prefab_id: &str, body: &str) -> Vec<u8> {
    let Ok(path) = prefab_path_for_id(prefab_id) else {
        return json_response(400, &json!({"ok": false, "error": "invalid prefab id"}));
    };
    let prefab = match parse_prefab_body(body) {
        Ok(prefab) => prefab,
        Err(error) => return json_response(400, &json!({"ok": false, "errors": [error]})),
    };
    if let Err(errors) = prefab.validate() {
        return json_response(422, &json!({"ok": false, "errors": errors}));
    }

    let backup = match backup_prefab_file(&path) {
        Ok(backup) => backup,
        Err(error) => return json_response(500, &json!({"ok": false, "error": error})),
    };
    match prefab.save_to_path(&path) {
        Ok(()) => json_response(
            200,
            &json!({
                "ok": true,
                "id": prefab_id.trim_end_matches(".json"),
                "path": path.to_string_lossy().replace('\\', "/"),
                "backup": backup.map(|path| path.to_string_lossy().replace('\\', "/")),
            }),
        ),
        Err(error) => json_response(500, &json!({"ok": false, "error": error})),
    }
}

fn delete_prefab_response(prefab_id: &str) -> Vec<u8> {
    let Ok(path) = prefab_path_for_id(prefab_id) else {
        return json_response(400, &json!({"ok": false, "error": "invalid prefab id"}));
    };
    if !path.is_file() {
        return json_response(404, &json!({"ok": false, "error": "prefab does not exist"}));
    }
    let backup = match backup_prefab_file(&path) {
        Ok(Some(backup)) => backup,
        Ok(None) => {
            return json_response(404, &json!({"ok": false, "error": "prefab does not exist"}));
        }
        Err(error) => return json_response(500, &json!({"ok": false, "error": error})),
    };
    match fs::remove_file(&path) {
        Ok(()) => json_response(
            200,
            &json!({
                "ok": true,
                "id": prefab_id.trim_end_matches(".json"),
                "backup": backup.to_string_lossy().replace('\\', "/"),
            }),
        ),
        Err(error) => json_response(
            500,
            &json!({"ok": false, "error": format!("failed to delete prefab: {}", error)}),
        ),
    }
}

fn backup_level_file(path: &Path) -> Result<Option<PathBuf>, String> {
    backup_json_file(path, &Path::new("levels").join(".editor_backups"), "level")
}

fn backup_prefab_file(path: &Path) -> Result<Option<PathBuf>, String> {
    backup_json_file(
        path,
        &Path::new("prefabs").join(".editor_backups"),
        "prefab",
    )
}

fn backup_json_file(path: &Path, backup_dir: &Path, kind: &str) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("{} path has no file stem", kind))?;
    fs::create_dir_all(backup_dir)
        .map_err(|error| format!("failed to create editor backup directory: {}", error))?;
    let timestamp = backup_timestamp();
    let mut backup_path = backup_dir.join(format!("{}_{}.json", id, timestamp));
    let mut suffix = 2;
    while backup_path.exists() {
        backup_path = backup_dir.join(format!("{}_{}_{}.json", id, timestamp, suffix));
        suffix += 1;
    }
    fs::copy(path, &backup_path)
        .map_err(|error| format!("failed to write editor backup: {}", error))?;
    Ok(Some(backup_path))
}

fn parse_level_body(body: &str) -> Result<LevelData, String> {
    serde_json::from_str::<LevelData>(body)
        .map_err(|error| format!("failed to parse level JSON: {}", error))
}

fn parse_prefab_body(body: &str) -> Result<LevelPrefabData, String> {
    serde_json::from_str::<LevelPrefabData>(body)
        .map_err(|error| format!("failed to parse prefab JSON: {}", error))
}

fn level_path_for_id(level_id: &str) -> Result<PathBuf, String> {
    let id = level_id.trim_end_matches(".json");
    if !is_safe_content_id(id) {
        return Err(format!("invalid level id '{}'", level_id));
    }
    Ok(Path::new("levels").join(format!("{}.json", id)))
}

fn prefab_path_for_id(prefab_id: &str) -> Result<PathBuf, String> {
    let id = prefab_id.trim_end_matches(".json");
    if !is_safe_content_id(id) {
        return Err(format!("invalid prefab id '{}'", prefab_id));
    }
    Ok(Path::new("prefabs").join(format!("{}.json", id)))
}

fn is_safe_content_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn load_enemy_definitions() -> Vec<EnemyDefinition> {
    let mut definitions = load_toml_dir::<EnemyDefinition>("data/enemies");
    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    definitions
}

fn load_relic_definitions() -> Vec<RelicDefinition> {
    let mut definitions = load_toml_dir::<RelicDefinition>("data/relics");
    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    definitions
}

fn load_toml_dir<T>(dir: &str) -> Vec<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut values = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return values;
    };

    let mut paths: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && has_extension(path, "toml"))
        .collect();
    paths.sort();

    for path in paths {
        match fs::read_to_string(&path).and_then(|source| {
            toml::from_str::<T>(&source)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        }) {
            Ok(value) => values.push(value),
            Err(error) => eprintln!(
                "[EDITOR] Failed to read '{}': {}",
                path.to_string_lossy(),
                error
            ),
        }
    }

    values
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn backup_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn static_response(content_type: &str, body: &str) -> Vec<u8> {
    response(200, content_type, body.as_bytes().to_vec())
}

fn json_response(status: u16, value: &serde_json::Value) -> Vec<u8> {
    response(
        status,
        "application/json; charset=utf-8",
        serde_json::to_vec_pretty(value).unwrap_or_else(|_| b"{\"ok\":false}".to_vec()),
    )
}

fn response(status: u16, content_type: &str, body: Vec<u8>) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        422 => "Unprocessable Content",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(&body);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_paths_are_restricted_to_safe_ids() {
        assert_eq!(
            level_path_for_id("movement_test")
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            "levels/movement_test.json"
        );
        assert!(level_path_for_id("../save").is_err());
        assert!(level_path_for_id("bad/name").is_err());
        assert!(level_path_for_id("").is_err());
    }

    #[test]
    fn prefab_paths_are_restricted_to_safe_ids() {
        assert_eq!(
            prefab_path_for_id("basic_room")
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            "prefabs/basic_room.json"
        );
        assert!(prefab_path_for_id("../levels/movement_test").is_err());
        assert!(prefab_path_for_id("bad/name").is_err());
        assert!(prefab_path_for_id("").is_err());
    }

    #[test]
    fn header_end_finds_http_separator() {
        assert_eq!(find_header_end(b"GET / HTTP/1.1\r\n\r\nbody"), Some(14));
    }

    #[test]
    fn host_and_origin_are_limited_to_local_editor_port() {
        assert!(host_allowed(Some("127.0.0.1:4327"), 4327));
        assert!(host_allowed(Some("localhost:4327"), 4327));
        assert!(!host_allowed(Some("example.com:4327"), 4327));
        assert!(!host_allowed(Some("127.0.0.1:4328"), 4327));

        assert!(origin_allowed(None, 4327));
        assert!(origin_allowed(Some("http://127.0.0.1:4327"), 4327));
        assert!(origin_allowed(Some("http://localhost:4327"), 4327));
        assert!(!origin_allowed(Some("https://evil.example"), 4327));
    }

    #[test]
    fn api_token_check_requires_exact_session_token() {
        assert!(token_matches("abc123", "abc123"));
        assert!(!token_matches("abc123", "abc124"));
        assert!(!token_matches("abc123", "abc1234"));
    }
}
