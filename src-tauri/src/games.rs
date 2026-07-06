use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::db;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
    name: String,
    exe_path: String,
    folder_path: String,
    icon: Option<String>,
    cover: Option<String>,
    args: Option<String>,
    work_dir: Option<String>,
    favorite: bool,
    play_count: i64,
    last_play_time: Option<i64>,
    create_time: i64,
    update_time: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGamePayload {
    name: String,
    exe_path: String,
    work_dir: Option<String>,
    args: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGamePayload {
    id: String,
    name: String,
    exe_path: String,
    work_dir: Option<String>,
    args: Option<String>,
    favorite: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCandidate {
    name: String,
    exe_path: String,
    folder_path: String,
    exe_file_name: String,
    exists: bool,
}

pub fn list_games(app: &AppHandle) -> Result<Vec<Game>, String> {
    let connection = db::open_connection(app)?;
    let mut games = query_games(&connection)?;
    for game in &mut games {
        ensure_game_visual_assets(app, &connection, game)?;
    }

    Ok(games)
}

fn ensure_game_visual_assets(
    app: &AppHandle,
    connection: &Connection,
    game: &mut Game,
) -> Result<(), String> {
    let mut changed = false;

    if game.icon.is_none() {
        game.icon = extract_game_icon(app, &game.exe_path, &game.id)?;
        changed |= game.icon.is_some();
    }

    if game.cover.is_none() {
        game.cover = detect_and_cache_cover(app, &game.folder_path, &game.id)?;
        changed |= game.cover.is_some();
    }

    if changed {
        let now = current_timestamp_millis()?;
        connection
            .execute(
                "
                UPDATE games
                SET icon = ?1,
                    cover = ?2,
                    update_time = ?3
                WHERE id = ?4
                ",
                params![game.icon, game.cover, now, game.id],
            )
            .map_err(|error| error.to_string())?;
        game.update_time = now;
    }

    Ok(())
}

fn scan_games(app: &AppHandle, directory: &str) -> Result<Vec<ScanCandidate>, String> {
    let connection = db::open_connection(app)?;
    let root = normalize_existing_directory(directory)?;
    let existing_exe_paths = query_existing_exe_paths(&connection)?;
    let mut candidates = Vec::new();

    scan_directory(&root, &existing_exe_paths, &mut candidates)?;
    candidates.sort_by(|left, right| {
        left.exists
            .cmp(&right.exists)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok(candidates)
}

pub fn get_game(app: &AppHandle, id: &str) -> Result<Option<Game>, String> {
    let connection = db::open_connection(app)?;
    get_game_by_id(&connection, id)
}

pub fn create_game(app: &AppHandle, payload: CreateGamePayload) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let input = normalize_game_fields(
        payload.name,
        payload.exe_path,
        payload.work_dir,
        payload.args,
    )?;

    if exe_path_exists(&connection, &input.exe_path)? {
        return Err("该游戏启动路径已存在".to_string());
    }

    let now = current_timestamp_millis()?;
    let id = format!("game-{}", now);
    let icon = extract_game_icon(app, &input.exe_path, &id)?;
    let cover = detect_and_cache_cover(app, &input.folder_path, &id)?;

    connection
        .execute(
            "
            INSERT INTO games (
                id,
                name,
                exe_path,
                folder_path,
                icon,
                cover,
                args,
                work_dir,
                favorite,
                play_count,
                last_play_time,
                create_time,
                update_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, NULL, ?9, ?10)
            ",
            params![
                id,
                input.name,
                input.exe_path,
                input.folder_path,
                icon,
                cover,
                input.args,
                input.work_dir,
                now,
                now
            ],
        )
        .map_err(|error| error.to_string())?;

    get_game_by_id(&connection, &id)?.ok_or_else(|| "游戏创建后无法读取".to_string())
}

pub fn update_game(app: &AppHandle, payload: UpdateGamePayload) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let id = payload.id.trim().to_string();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    let existing_game =
        get_game_by_id(&connection, &id)?.ok_or_else(|| "游戏不存在或已被删除".to_string())?;

    let input = normalize_game_fields(
        payload.name,
        payload.exe_path,
        payload.work_dir,
        payload.args,
    )?;
    if exe_path_exists_for_other_game(&connection, &input.exe_path, &id)? {
        return Err("该游戏启动路径已存在".to_string());
    }

    let now = current_timestamp_millis()?;
    let icon = if existing_game.exe_path != input.exe_path || existing_game.icon.is_none() {
        extract_game_icon(app, &input.exe_path, &id)?.or(existing_game.icon)
    } else {
        existing_game.icon
    };
    let cover = if existing_game.folder_path != input.folder_path || existing_game.cover.is_none() {
        detect_and_cache_cover(app, &input.folder_path, &id)?.or(existing_game.cover)
    } else {
        existing_game.cover
    };

    connection
        .execute(
            "
            UPDATE games
            SET name = ?1,
                exe_path = ?2,
                folder_path = ?3,
                icon = ?4,
                cover = ?5,
                args = ?6,
                work_dir = ?7,
                favorite = ?8,
                update_time = ?9
            WHERE id = ?10
            ",
            params![
                input.name,
                input.exe_path,
                input.folder_path,
                icon,
                cover,
                input.args,
                input.work_dir,
                i64::from(payload.favorite),
                now,
                id
            ],
        )
        .map_err(|error| error.to_string())?;

    get_game_by_id(&connection, &id)?.ok_or_else(|| "游戏更新后无法读取".to_string())
}
pub fn delete_game(app: &AppHandle, id: &str) -> Result<(), String> {
    let connection = db::open_connection(app)?;
    let id = id.trim();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    let affected_rows = connection
        .execute("DELETE FROM games WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;

    if affected_rows == 0 {
        return Err("游戏不存在或已被删除".to_string());
    }

    Ok(())
}

pub fn launch_game(app: &AppHandle, id: &str) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let id = id.trim();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    let game =
        get_game_by_id(&connection, id)?.ok_or_else(|| "游戏不存在或已被删除".to_string())?;
    let exe_path = normalize_existing_exe_path(&game.exe_path)?;
    let work_dir = match game.work_dir.as_deref() {
        Some(value) if !value.trim().is_empty() => normalize_existing_directory(value)?,
        _ => exe_path
            .parent()
            .ok_or_else(|| "无法识别游戏所在目录".to_string())?
            .to_path_buf(),
    };

    let mut command = Command::new(&exe_path);
    command.current_dir(work_dir);
    if let Some(args) = game.args.as_deref() {
        command.args(parse_launch_args(args)?);
    }

    command
        .spawn()
        .map_err(|error| format!("启动游戏失败：{error}"))?;

    let now = current_timestamp_millis()?;
    connection
        .execute(
            "
            UPDATE games
            SET play_count = play_count + 1,
                last_play_time = ?1,
                update_time = ?2
            WHERE id = ?3
            ",
            params![now, now, id],
        )
        .map_err(|error| error.to_string())?;

    get_game_by_id(&connection, id)?.ok_or_else(|| "游戏启动后无法读取".to_string())
}

fn parse_launch_args(args: &str) -> Result<Vec<String>, String> {
    let mut parsed = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaping = false;

    for character in args.chars() {
        if escaping {
            current.push(character);
            escaping = false;
            continue;
        }

        match character {
            '\\' if in_quotes => escaping = true,
            '"' => in_quotes = !in_quotes,
            value if value.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parsed.push(std::mem::take(&mut current));
                }
            }
            value => current.push(value),
        }
    }

    if escaping {
        current.push('\\');
    }
    if in_quotes {
        return Err("启动参数中的引号未闭合".to_string());
    }
    if !current.is_empty() {
        parsed.push(current);
    }

    Ok(parsed)
}

fn normalize_game_fields(
    name: String,
    exe_path: String,
    work_dir: Option<String>,
    args: Option<String>,
) -> Result<NormalizedGameFields, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("游戏名称不能为空".to_string());
    }

    let exe_path = normalize_existing_exe_path(&exe_path)?;
    let folder_path = exe_path
        .parent()
        .ok_or_else(|| "无法识别游戏所在目录".to_string())?
        .to_path_buf();
    let work_dir = match work_dir.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => normalize_existing_directory(&value)?,
        _ => folder_path.clone(),
    };

    Ok(NormalizedGameFields {
        name,
        exe_path: path_to_string(exe_path)?,
        folder_path: path_to_string(folder_path)?,
        work_dir: Some(path_to_string(work_dir)?),
        args: args.and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        }),
    })
}

fn game_asset_dir(app: &AppHandle, game_id: &str) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("assets")
        .join("games")
        .join(game_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory)
}

fn detect_and_cache_cover(
    app: &AppHandle,
    folder_path: &str,
    game_id: &str,
) -> Result<Option<String>, String> {
    let folder = PathBuf::from(folder_path);
    let Some(source) = find_cover_candidate(&folder) else {
        return Ok(None);
    };

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let target = game_asset_dir(app, game_id)?.join(format!("cover.{extension}"));
    fs::copy(&source, &target).map_err(|error| error.to_string())?;

    path_to_string(target).map(Some)
}

fn find_cover_candidate(folder: &Path) -> Option<PathBuf> {
    const FILE_STEMS: &[&str] = &[
        "cover",
        "poster",
        "capsule",
        "header",
        "library",
        "background",
        "hero",
    ];
    const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];
    const SUBDIRECTORIES: &[&str] = &[".", "images", "image", "assets", "media", "launcher"];

    for subdirectory in SUBDIRECTORIES {
        let directory = if *subdirectory == "." {
            folder.to_path_buf()
        } else {
            folder.join(subdirectory)
        };
        if !directory.is_dir() {
            continue;
        }

        for stem in FILE_STEMS {
            for extension in EXTENSIONS {
                let candidate = directory.join(format!("{stem}.{extension}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn extract_game_icon(
    app: &AppHandle,
    exe_path: &str,
    game_id: &str,
) -> Result<Option<String>, String> {
    extract_game_icon_for_platform(app, exe_path, game_id)
}

#[cfg(target_os = "windows")]
fn extract_game_icon_for_platform(
    app: &AppHandle,
    exe_path: &str,
    game_id: &str,
) -> Result<Option<String>, String> {
    let target = game_asset_dir(app, game_id)?.join("icon.ico");
    let command = format!(
        "Add-Type -AssemblyName System.Drawing; \
         $icon=[System.Drawing.Icon]::ExtractAssociatedIcon('{exe_path}'); \
         if ($null -eq $icon) {{ exit 2 }}; \
         $stream=[System.IO.File]::Open('{target_path}', [System.IO.FileMode]::Create); \
         try {{ $icon.Save($stream) }} finally {{ $stream.Dispose(); $icon.Dispose() }}",
        exe_path = escape_powershell_single_quoted(exe_path),
        target_path = escape_powershell_single_quoted(&path_to_string(target.clone())?),
    );

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command,
        ])
        .status()
        .map_err(|error| error.to_string())?;

    if status.success() && target.is_file() {
        path_to_string(target).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_game_icon_for_platform(
    _app: &AppHandle,
    _exe_path: &str,
    _game_id: &str,
) -> Result<Option<String>, String> {
    Ok(None)
}

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn normalize_existing_exe_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path.trim());
    if !path.exists() {
        return Err("启动程序不存在".to_string());
    }
    if !path.is_file() {
        return Err("启动路径必须指向文件".to_string());
    }
    if !has_exe_extension(&path) {
        return Err("启动路径必须是 .exe 文件".to_string());
    }
    path.canonicalize().map_err(|error| error.to_string())
}

fn normalize_existing_directory(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path.trim());
    if !path.exists() {
        return Err("工作目录不存在".to_string());
    }
    if !path.is_dir() {
        return Err("工作目录必须是文件夹".to_string());
    }
    path.canonicalize().map_err(|error| error.to_string())
}

fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

fn scan_directory(
    directory: &Path,
    existing_exe_paths: &HashSet<String>,
    candidates: &mut Vec<ScanCandidate>,
) -> Result<(), String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            if should_skip_directory(&path) {
                continue;
            }
            scan_directory(&path, existing_exe_paths, candidates)?;
            continue;
        }

        if !path.is_file() || !has_exe_extension(&path) || should_skip_exe(&path) {
            continue;
        }

        let exe_path = path_to_string(path.canonicalize().map_err(|error| error.to_string())?)?;
        let folder_path = path
            .parent()
            .ok_or_else(|| "无法识别候选程序所在目录".to_string())?
            .to_path_buf();
        let exe_file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "无法识别候选程序文件名".to_string())?
            .to_string();

        candidates.push(ScanCandidate {
            name: infer_game_name(&path),
            exists: existing_exe_paths.contains(&exe_path),
            exe_path,
            folder_path: path_to_string(
                folder_path
                    .canonicalize()
                    .map_err(|error| error.to_string())?,
            )?,
            exe_file_name,
        });
    }

    Ok(())
}

fn should_skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "$recycle.bin"
                    | ".git"
                    | "node_modules"
                    | "redist"
                    | "redistributable"
                    | "redistributables"
                    | "_commonredist"
                    | "directx"
                    | "vcredist"
            )
        })
}

fn should_skip_exe(path: &Path) -> bool {
    path.file_stem()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            let name = name.to_ascii_lowercase();
            name.contains("unins")
                || name.contains("uninstall")
                || name.contains("crash")
                || name.contains("reporter")
                || name.contains("redist")
                || name.contains("vcredist")
                || name.contains("dxsetup")
                || name.contains("eac")
                || name.contains("easyanticheat")
        })
}

fn infer_game_name(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map_or_else(
            || {
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Game")
                    .to_string()
            },
            ToString::to_string,
        )
}

fn path_to_string(path: PathBuf) -> Result<String, String> {
    let path = path
        .into_os_string()
        .into_string()
        .map_err(|_| "路径包含无效 Unicode 字符".to_string())?;

    Ok(strip_windows_extended_path_prefix(path))
}

fn strip_windows_extended_path_prefix(path: String) -> String {
    if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", stripped);
    }

    path.strip_prefix(r"\\?\")
        .map_or(path.clone(), ToString::to_string)
}

fn exe_path_exists(connection: &Connection, exe_path: &str) -> Result<bool, String> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(1) FROM games WHERE exe_path = ?1",
            params![exe_path],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    Ok(count > 0)
}

fn query_existing_exe_paths(connection: &Connection) -> Result<HashSet<String>, String> {
    let mut statement = connection
        .prepare("SELECT exe_path FROM games")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;

    rows.map(|row| row.map(strip_windows_extended_path_prefix))
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())
}

fn exe_path_exists_for_other_game(
    connection: &Connection,
    exe_path: &str,
    id: &str,
) -> Result<bool, String> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(1) FROM games WHERE exe_path = ?1 AND id <> ?2",
            params![exe_path, id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    Ok(count > 0)
}

fn get_game_by_id(connection: &Connection, id: &str) -> Result<Option<Game>, String> {
    connection
        .query_row(
            "
            SELECT
                id,
                name,
                exe_path,
                folder_path,
                icon,
                cover,
                args,
                work_dir,
                favorite,
                play_count,
                last_play_time,
                create_time,
                update_time
            FROM games
            WHERE id = ?1
            ",
            params![id],
            map_game_row,
        )
        .optional()
        .map_err(|error| error.to_string())
}

fn query_games(connection: &Connection) -> Result<Vec<Game>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
                id,
                name,
                exe_path,
                folder_path,
                icon,
                cover,
                args,
                work_dir,
                favorite,
                play_count,
                last_play_time,
                create_time,
                update_time
            FROM games
            ORDER BY create_time DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], map_game_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn map_game_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        name: row.get(1)?,
        exe_path: strip_windows_extended_path_prefix(row.get(2)?),
        folder_path: strip_windows_extended_path_prefix(row.get(3)?),
        icon: row.get(4)?,
        cover: row.get(5)?,
        args: row.get(6)?,
        work_dir: row
            .get::<_, Option<String>>(7)?
            .map(strip_windows_extended_path_prefix),
        favorite: row.get::<_, i64>(8)? != 0,
        play_count: row.get(9)?,
        last_play_time: row.get(10)?,
        create_time: row.get(11)?,
        update_time: row.get(12)?,
    })
}

fn current_timestamp_millis() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;

    i64::try_from(duration.as_millis()).map_err(|_| "当前时间戳超出范围".to_string())
}

struct NormalizedGameFields {
    name: String,
    exe_path: String,
    folder_path: String,
    work_dir: Option<String>,
    args: Option<String>,
}

#[tauri::command]
pub fn list_games_command(app: AppHandle) -> Result<Vec<Game>, String> {
    list_games(&app)
}

#[tauri::command]
pub fn get_game_command(app: AppHandle, id: String) -> Result<Option<Game>, String> {
    get_game(&app, &id)
}

#[tauri::command]
pub fn create_game_command(app: AppHandle, payload: CreateGamePayload) -> Result<Game, String> {
    create_game(&app, payload)
}

#[tauri::command]
pub fn update_game_command(app: AppHandle, payload: UpdateGamePayload) -> Result<Game, String> {
    update_game(&app, payload)
}

#[tauri::command]
pub fn delete_game_command(app: AppHandle, id: String) -> Result<(), String> {
    delete_game(&app, &id)
}

#[tauri::command]
pub fn launch_game_command(app: AppHandle, id: String) -> Result<Game, String> {
    launch_game(&app, &id)
}

#[tauri::command]
pub async fn scan_games_command(
    app: AppHandle,
    directory: String,
) -> Result<Vec<ScanCandidate>, String> {
    tauri::async_runtime::spawn_blocking(move || scan_games(&app, &directory))
        .await
        .map_err(|error| error.to_string())?
}
