mod assets;
pub(crate) mod commands;
#[allow(dead_code)]
pub(crate) mod covers;
mod launcher;
mod models;
mod repository;
mod scanner;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tauri::AppHandle;

use crate::db;

use covers::models::CoverSelection;
use covers::provider::CoverProvider;
use models::{CreateGamePayload, Game, NormalizedGameFields, UpdateGamePayload};

fn list_games(app: &AppHandle) -> Result<Vec<Game>, String> {
    let connection = db::open_connection(app)?;
    let mut games = repository::list(&connection)?;
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

    if assets::should_refresh_icon(game.icon.as_deref()) {
        if let Some(icon) = assets::extract_game_icon(app, &game.exe_path, &game.id)? {
            game.icon = Some(icon);
            changed = true;
        }
    }
    if game.cover.is_none() {
        game.cover = assets::detect_and_cache_cover(app, &game.folder_path, &game.id)?;
        changed |= game.cover.is_some();
    }

    if changed {
        game.update_time = current_timestamp_millis()?;
        repository::update_visual_assets(connection, game)?;
    }
    Ok(())
}

fn get_game(app: &AppHandle, id: &str) -> Result<Option<Game>, String> {
    let connection = db::open_connection(app)?;
    repository::get_by_id(&connection, id)
}

async fn create_game(app: &AppHandle, payload: CreateGamePayload) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let input = normalize_game_fields(
        payload.name,
        payload.exe_path,
        payload.work_dir,
        payload.args,
    )?;
    if repository::exe_path_exists(&connection, &input.exe_path)? {
        return Err("该游戏启动路径已存在".to_string());
    }
    drop(connection);

    let now = current_timestamp_millis()?;
    let id = format!("game-{now}");
    let icon = assets::extract_game_icon(app, &input.exe_path, &id)?;
    let cover = resolve_created_cover(
        app,
        &id,
        &input.folder_path,
        payload.cover_selection,
        payload.cover_path,
    )
    .await?;
    let game = Game {
        id: id.clone(),
        name: input.name,
        exe_path: input.exe_path,
        folder_path: input.folder_path,
        icon,
        cover,
        args: input.args,
        work_dir: input.work_dir,
        favorite: false,
        favorite_time: None,
        play_count: 0,
        last_play_time: None,
        create_time: now,
        update_time: now,
    };

    let connection = db::open_connection(app)?;
    repository::insert(&connection, &game)?;
    assets::cleanup_stale_cover_files(app, &id, game.cover.as_deref());
    repository::get_by_id(&connection, &id)?.ok_or_else(|| "游戏创建后无法读取".to_string())
}

async fn update_game(app: &AppHandle, payload: UpdateGamePayload) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let id = payload.id.trim().to_string();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    let existing = repository::get_by_id(&connection, &id)?
        .ok_or_else(|| "游戏不存在或已被删除".to_string())?;
    let input = normalize_game_fields(
        payload.name,
        payload.exe_path,
        payload.work_dir,
        payload.args,
    )?;
    if repository::exe_path_exists_for_other_game(&connection, &input.exe_path, &id)? {
        return Err("该游戏启动路径已存在".to_string());
    }
    drop(connection);

    let now = current_timestamp_millis()?;
    let favorite_time = match (existing.favorite, payload.favorite) {
        (false, true) => Some(now),
        (true, true) => existing.favorite_time,
        _ => None,
    };
    let icon = if existing.exe_path != input.exe_path || existing.icon.is_none() {
        assets::extract_game_icon(app, &input.exe_path, &id)?.or(existing.icon.clone())
    } else {
        existing.icon.clone()
    };
    let cover = resolve_updated_cover(
        app,
        &id,
        &input.folder_path,
        &existing,
        payload.cover_selection,
        payload.cover_path,
    )
    .await?;
    let game = Game {
        id: id.clone(),
        name: input.name,
        exe_path: input.exe_path,
        folder_path: input.folder_path,
        icon,
        cover,
        args: input.args,
        work_dir: input.work_dir,
        favorite: payload.favorite,
        favorite_time,
        play_count: existing.play_count,
        last_play_time: existing.last_play_time,
        create_time: existing.create_time,
        update_time: now,
    };

    let connection = db::open_connection(app)?;
    repository::update_metadata(&connection, &game)?;
    assets::cleanup_stale_cover_files(app, &id, game.cover.as_deref());
    repository::get_by_id(&connection, &id)?.ok_or_else(|| "游戏更新后无法读取".to_string())
}

fn delete_game(app: &AppHandle, id: &str) -> Result<(), String> {
    let connection = db::open_connection(app)?;
    let id = id.trim();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }
    if !repository::delete(&connection, id)? {
        return Err("游戏不存在或已被删除".to_string());
    }
    Ok(())
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

fn normalize_optional_path(path: Option<String>) -> Option<String> {
    path.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

async fn resolve_created_cover(
    app: &AppHandle,
    game_id: &str,
    folder_path: &str,
    selection: Option<CoverSelection>,
    legacy_cover_path: Option<String>,
) -> Result<Option<String>, String> {
    match selection {
        Some(CoverSelection::Local { path }) => {
            Ok(Some(assets::cache_manual_cover(app, &path, game_id)?))
        }
        Some(CoverSelection::Remote {
            provider,
            provider_game_id,
            asset_id,
        }) => cache_remote_cover(app, game_id, &provider, &provider_game_id, &asset_id)
            .await
            .map(Some),
        Some(CoverSelection::Remove) => Ok(None),
        Some(CoverSelection::Unchanged) | None => {
            match normalize_optional_path(legacy_cover_path) {
                Some(path) => Ok(Some(assets::cache_manual_cover(app, &path, game_id)?)),
                None => assets::detect_and_cache_cover(app, folder_path, game_id),
            }
        }
    }
}

async fn resolve_updated_cover(
    app: &AppHandle,
    game_id: &str,
    folder_path: &str,
    existing: &Game,
    selection: Option<CoverSelection>,
    legacy_cover_path: Option<String>,
) -> Result<Option<String>, String> {
    match selection {
        Some(CoverSelection::Local { path }) => {
            Ok(Some(assets::cache_manual_cover(app, &path, game_id)?))
        }
        Some(CoverSelection::Remote {
            provider,
            provider_game_id,
            asset_id,
        }) => cache_remote_cover(app, game_id, &provider, &provider_game_id, &asset_id)
            .await
            .map(Some),
        Some(CoverSelection::Remove) => Ok(None),
        Some(CoverSelection::Unchanged) => Ok(existing.cover.clone()),
        None => match normalize_optional_path(legacy_cover_path) {
            Some(path) => Ok(Some(assets::cache_manual_cover(app, &path, game_id)?)),
            None if existing.folder_path != folder_path || existing.cover.is_none() => {
                Ok(assets::detect_and_cache_cover(app, folder_path, game_id)?
                    .or(existing.cover.clone()))
            }
            None => Ok(existing.cover.clone()),
        },
    }
}

async fn cache_remote_cover(
    app: &AppHandle,
    game_id: &str,
    provider_id: &str,
    provider_game_id: &str,
    asset_id: &str,
) -> Result<String, String> {
    let provider = crate::settings::online_cover_provider(app)?;
    if provider_id.trim() != provider.provider_id() {
        return Err("不支持的联网封面数据源".to_string());
    }
    let asset = provider
        .resolve_cover(provider_game_id.trim(), asset_id.trim())
        .await
        .map_err(|error| crate::settings::record_online_cover_provider_error(app, error))?
        .ok_or_else(|| "选择的联网封面已失效，请重新搜索".to_string())?;
    let bytes = covers::download::download_cover(&asset.download_url).await?;
    assets::cache_remote_cover(app, bytes, game_id)
}

pub(super) fn normalize_existing_exe_path(path: &str) -> Result<PathBuf, String> {
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

pub(super) fn normalize_existing_directory(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path.trim());
    if !path.exists() {
        return Err("工作目录不存在".to_string());
    }
    if !path.is_dir() {
        return Err("工作目录必须是文件夹".to_string());
    }
    path.canonicalize().map_err(|error| error.to_string())
}

pub(super) fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

pub(super) fn path_to_string(path: PathBuf) -> Result<String, String> {
    let path = path
        .into_os_string()
        .into_string()
        .map_err(|_| "路径包含无效 Unicode 字符".to_string())?;
    Ok(strip_windows_extended_path_prefix(path))
}

pub(super) fn strip_windows_extended_path_prefix(path: String) -> String {
    if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", stripped);
    }
    path.strip_prefix(r"\\?\")
        .map_or(path.clone(), ToString::to_string)
}

pub(super) fn current_timestamp_millis() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    i64::try_from(duration.as_millis()).map_err(|_| "当前时间戳超出范围".to_string())
}
