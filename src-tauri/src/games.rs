use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{ColorType, DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::db;

const MAX_COVER_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_COVER_DIMENSION: u32 = 8192;
const MAX_COVER_PIXELS: u64 = 40_000_000;
const MAX_CACHED_COVER_DIMENSION: u32 = 2400;
const COVER_JPEG_QUALITY: u8 = 88;

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
    favorite_time: Option<i64>,
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
    cover_path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGamePayload {
    id: String,
    name: String,
    exe_path: String,
    work_dir: Option<String>,
    args: Option<String>,
    cover_path: Option<String>,
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

    if should_refresh_icon(game.icon.as_deref()) {
        if let Some(icon) = extract_game_icon(app, &game.exe_path, &game.id)? {
            game.icon = Some(icon);
            changed = true;
        }
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
    let cover = match normalize_optional_path(payload.cover_path) {
        Some(path) => Some(cache_manual_cover(app, &path, &id)?),
        None => detect_and_cache_cover(app, &input.folder_path, &id)?,
    };

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
                favorite_time,
                play_count,
                last_play_time,
                create_time,
                update_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, NULL, 0, NULL, ?9, ?10)
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

    cleanup_stale_cover_files(app, &id, cover.as_deref());

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
    let favorite_time = match (existing_game.favorite, payload.favorite) {
        (false, true) => Some(now),
        (true, true) => existing_game.favorite_time,
        _ => None,
    };
    let icon = if existing_game.exe_path != input.exe_path || existing_game.icon.is_none() {
        extract_game_icon(app, &input.exe_path, &id)?.or(existing_game.icon)
    } else {
        existing_game.icon
    };
    let cover = match normalize_optional_path(payload.cover_path) {
        Some(path) => Some(cache_manual_cover(app, &path, &id)?),
        None if existing_game.folder_path != input.folder_path || existing_game.cover.is_none() => {
            detect_and_cache_cover(app, &input.folder_path, &id)?.or(existing_game.cover)
        }
        None => existing_game.cover,
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
                favorite_time = ?9,
                update_time = ?10
            WHERE id = ?11
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
                favorite_time,
                now,
                id
            ],
        )
        .map_err(|error| error.to_string())?;

    cleanup_stale_cover_files(app, &id, cover.as_deref());

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
    let mut argument_started = false;
    let mut characters = args.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                argument_started = true;
            }
            '\\' => {
                let mut backslash_count = 1;
                while characters.peek() == Some(&'\\') {
                    characters.next();
                    backslash_count += 1;
                }

                if characters.peek() == Some(&'"') {
                    current.extend(std::iter::repeat_n('\\', backslash_count / 2));
                    characters.next();
                    if backslash_count % 2 == 0 {
                        in_quotes = !in_quotes;
                    } else {
                        current.push('"');
                    }
                } else {
                    current.extend(std::iter::repeat_n('\\', backslash_count));
                }
                argument_started = true;
            }
            value if value.is_whitespace() && !in_quotes => {
                if argument_started {
                    parsed.push(std::mem::take(&mut current));
                    argument_started = false;
                }
            }
            value => {
                current.push(value);
                argument_started = true;
            }
        }
    }

    if in_quotes {
        return Err("启动参数中的引号未闭合".to_string());
    }
    if argument_started {
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

fn cache_manual_cover(app: &AppHandle, source_path: &str, game_id: &str) -> Result<String, String> {
    let source = PathBuf::from(source_path.trim());
    if !source.is_file() {
        return Err("选择的封面文件不存在".to_string());
    }

    let timestamp = current_timestamp_millis()?;
    cache_cover_image(app, &source, game_id, &format!("cover-manual-{timestamp}"))
}

fn cache_cover_image(
    app: &AppHandle,
    source: &Path,
    game_id: &str,
    target_stem: &str,
) -> Result<String, String> {
    let metadata = source
        .metadata()
        .map_err(|error| format!("读取封面文件失败：{error}"))?;
    if metadata.len() > MAX_COVER_FILE_SIZE_BYTES {
        return Err("封面文件不能超过 10 MB".to_string());
    }

    let bytes = fs::read(source).map_err(|error| format!("读取封面文件失败：{error}"))?;
    let image = decode_cover_image(bytes)?;

    encode_cached_cover(app, game_id, target_stem, image)
}

fn decode_cover_image(bytes: Vec<u8>) -> Result<DynamicImage, String> {
    let reader = ImageReader::new(Cursor::new(bytes.as_slice()))
        .with_guessed_format()
        .map_err(|error| format!("识别封面格式失败：{error}"))?;
    let format = reader
        .format()
        .filter(|format| {
            matches!(
                format,
                ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
            )
        })
        .ok_or_else(|| "封面仅支持 PNG、JPEG 或 WebP 格式".to_string())?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|error| format!("读取封面尺寸失败：{error}"))?;
    validate_cover_dimensions(width, height)?;

    let mut decoder = ImageReader::with_format(Cursor::new(bytes), format)
        .into_decoder()
        .map_err(|error| format!("创建封面解码器失败：{error}"))?;
    let orientation = decoder
        .orientation()
        .map_err(|error| format!("读取封面方向失败：{error}"))?;
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| format!("解码封面图片失败：{error}"))?;
    image.apply_orientation(orientation);

    if image.width() > MAX_CACHED_COVER_DIMENSION || image.height() > MAX_CACHED_COVER_DIMENSION {
        Ok(image.resize(
            MAX_CACHED_COVER_DIMENSION,
            MAX_CACHED_COVER_DIMENSION,
            FilterType::Lanczos3,
        ))
    } else {
        Ok(image)
    }
}

fn validate_cover_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("封面图片尺寸无效".to_string());
    }
    if width > MAX_COVER_DIMENSION || height > MAX_COVER_DIMENSION {
        return Err(format!("封面图片宽高不能超过 {MAX_COVER_DIMENSION} 像素"));
    }
    if u64::from(width) * u64::from(height) > MAX_COVER_PIXELS {
        return Err("封面图片总像素不能超过 4000 万".to_string());
    }

    Ok(())
}

fn encode_cached_cover(
    app: &AppHandle,
    game_id: &str,
    target_stem: &str,
    image: DynamicImage,
) -> Result<String, String> {
    let has_alpha = image.color().has_alpha();
    let extension = if has_alpha { "webp" } else { "jpg" };
    let target = game_asset_dir(app, game_id)?.join(format!("{target_stem}.{extension}"));
    let temporary = target.with_extension(format!("{extension}.tmp"));
    let width = image.width();
    let height = image.height();

    let encode_result = (|| -> Result<(), String> {
        let file =
            File::create(&temporary).map_err(|error| format!("创建封面缓存文件失败：{error}"))?;
        let writer = BufWriter::new(file);

        if has_alpha {
            let rgba = image.to_rgba8();
            WebPEncoder::new_lossless(writer)
                .write_image(rgba.as_raw(), width, height, ColorType::Rgba8.into())
                .map_err(|error| format!("编码 WebP 封面失败：{error}"))
        } else {
            let rgb = image.to_rgb8();
            JpegEncoder::new_with_quality(writer, COVER_JPEG_QUALITY)
                .write_image(rgb.as_raw(), width, height, ColorType::Rgb8.into())
                .map_err(|error| format!("编码 JPEG 封面失败：{error}"))
        }
    })();

    if let Err(error) = encode_result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }

    fs::rename(&temporary, &target).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("保存封面缓存失败：{error}")
    })?;

    path_to_string(target)
}

fn cleanup_stale_cover_files(app: &AppHandle, game_id: &str, current_cover: Option<&str>) {
    let Ok(directory) = game_asset_dir(app, game_id) else {
        return;
    };
    let current_cover = current_cover.map(PathBuf::from);
    let Ok(entries) = fs::read_dir(&directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_cover = path
            .file_stem()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                value == "cover"
                    || value.starts_with("cover-auto-")
                    || value.starts_with("cover-manual-")
            });
        if is_cover
            && current_cover
                .as_ref()
                .is_none_or(|current| current != &path)
        {
            let _ = fs::remove_file(path);
        }
    }
}

fn normalize_optional_path(path: Option<String>) -> Option<String> {
    path.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
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

    let Ok(timestamp) = current_timestamp_millis() else {
        return Ok(None);
    };
    match cache_cover_image(app, &source, game_id, &format!("cover-auto-{timestamp}")) {
        Ok(path) => Ok(Some(path)),
        Err(_) => Ok(None),
    }
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

fn should_refresh_icon(icon: Option<&str>) -> bool {
    match icon {
        None => true,
        Some(path) => Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ico")),
    }
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
    let target = game_asset_dir(app, game_id)?.join("icon.png");
    let Some(icon) = extract_best_icon_handle(Path::new(exe_path)) else {
        return Ok(None);
    };

    let render_result = render_icon_to_png(icon, &target, 256);
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon);
    }

    if render_result.is_ok() && target.is_file() {
        path_to_string(target).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn extract_best_icon_handle(
    path: &Path,
) -> Option<windows_sys::Win32::UI::WindowsAndMessaging::HICON> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::PrivateExtractIconsW;

    const ICON_SIZES: &[i32] = &[256, 128, 64, 48, 32];

    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    for size in ICON_SIZES {
        let mut icon = std::ptr::null_mut();
        let mut icon_id = 0;
        let count = unsafe {
            PrivateExtractIconsW(
                wide_path.as_ptr(),
                0,
                *size,
                *size,
                &mut icon,
                &mut icon_id,
                1,
                0,
            )
        };

        if count > 0 && !icon.is_null() {
            return Some(icon);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn render_icon_to_png(
    icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
    target: &Path,
    size: i32,
) -> Result<(), String> {
    use std::ffi::c_void;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

    let hdc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if hdc.is_null() {
        return Err("无法创建图标渲染上下文".to_string());
    }

    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default(); 1],
    };

    let bitmap = unsafe {
        CreateDIBSection(
            hdc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        )
    };
    if bitmap.is_null() || bits.is_null() {
        unsafe {
            DeleteDC(hdc);
        }
        return Err("无法创建图标位图".to_string());
    }

    let previous = unsafe { SelectObject(hdc, bitmap) };
    let byte_len = (size * size * 4) as usize;
    unsafe {
        std::ptr::write_bytes(bits, 0, byte_len);
    }

    let drawn = unsafe {
        DrawIconEx(
            hdc,
            0,
            0,
            icon,
            size,
            size,
            0,
            std::ptr::null_mut(),
            DI_NORMAL,
        )
    };
    if drawn == 0 {
        unsafe {
            SelectObject(hdc, previous);
            DeleteObject(bitmap);
            DeleteDC(hdc);
        }
        return Err("无法渲染图标".to_string());
    }

    let bgra = unsafe { std::slice::from_raw_parts(bits as *const u8, byte_len) };
    let rgba = bgra_to_rgba_with_alpha_fallback(bgra);

    unsafe {
        SelectObject(hdc, previous);
        DeleteObject(bitmap);
        DeleteDC(hdc);
    }

    write_png(target, size as u32, size as u32, &rgba)
}

#[cfg(target_os = "windows")]
fn bgra_to_rgba_with_alpha_fallback(bgra: &[u8]) -> Vec<u8> {
    let has_visible_alpha = bgra.chunks_exact(4).any(|pixel| pixel[3] != 0);
    let has_color = bgra
        .chunks_exact(4)
        .any(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0);

    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        let alpha = if has_visible_alpha || !has_color {
            pixel[3]
        } else if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
            255
        } else {
            0
        };
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], alpha]);
    }

    rgba
}
#[cfg(target_os = "windows")]
fn write_png(target: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let file = fs::File::create(target).map_err(|error| error.to_string())?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().map_err(|error| error.to_string())?;
    png_writer
        .write_image_data(rgba)
        .map_err(|error| error.to_string())
}
#[cfg(not(target_os = "windows"))]
fn extract_game_icon_for_platform(
    _app: &AppHandle,
    _exe_path: &str,
    _game_id: &str,
) -> Result<Option<String>, String> {
    Ok(None)
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
                favorite_time,
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
                favorite_time,
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
        favorite_time: row.get(9)?,
        play_count: row.get(10)?,
        last_play_time: row.get(11)?,
        create_time: row.get(12)?,
        update_time: row.get(13)?,
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{decode_cover_image, parse_launch_args, validate_cover_dimensions};
    use image::{DynamicImage, ImageFormat};

    #[test]
    fn parses_quoted_windows_path_without_removing_backslashes() {
        let parsed =
            parse_launch_args(r#""C:\Users\lsm\AppData\Local\Temp\game shift.txt""#).unwrap();

        assert_eq!(
            parsed,
            vec![r"C:\Users\lsm\AppData\Local\Temp\game shift.txt"]
        );
    }

    #[test]
    fn parses_command_with_quoted_argument() {
        let parsed =
            parse_launch_args(r#"/c "echo Game Shift args OK> %TEMP%\game-shift-args-test.txt""#)
                .unwrap();

        assert_eq!(
            parsed,
            vec![
                "/c",
                r"echo Game Shift args OK> %TEMP%\game-shift-args-test.txt"
            ]
        );
    }

    #[test]
    fn preserves_escaped_quotes_and_empty_arguments() {
        let parsed = parse_launch_args(r#"--message "say \"hello\"" --label """#).unwrap();

        assert_eq!(parsed, vec!["--message", "say \"hello\"", "--label", ""]);
    }

    #[test]
    fn rejects_unclosed_quotes() {
        let error = parse_launch_args(r#""C:\Games\example.exe"#).unwrap_err();

        assert_eq!(error, "启动参数中的引号未闭合");
    }

    #[test]
    fn decodes_and_resizes_valid_cover_image() {
        let source = DynamicImage::new_rgb8(3000, 1000);
        let mut encoded = Cursor::new(Vec::new());
        source.write_to(&mut encoded, ImageFormat::Png).unwrap();

        let decoded = decode_cover_image(encoded.into_inner()).unwrap();

        assert_eq!(decoded.width(), 2400);
        assert_eq!(decoded.height(), 800);
    }

    #[test]
    fn applies_exif_orientation_before_resizing() {
        let source = DynamicImage::new_rgb8(2, 1);
        let mut encoded = Cursor::new(Vec::new());
        source.write_to(&mut encoded, ImageFormat::Jpeg).unwrap();
        let mut jpeg = encoded.into_inner();

        let exif_orientation_rotate_90 = [
            0xff, 0xe1, 0x00, 0x22, b'E', b'x', b'i', b'f', 0x00, 0x00, b'I', b'I', 0x2a, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        jpeg.splice(2..2, exif_orientation_rotate_90);

        let decoded = decode_cover_image(jpeg).unwrap();

        assert_eq!(decoded.width(), 1);
        assert_eq!(decoded.height(), 2);
    }

    #[test]
    fn rejects_file_content_that_is_not_an_image() {
        let error = decode_cover_image(b"not an image".to_vec()).unwrap_err();

        assert_eq!(error, "封面仅支持 PNG、JPEG 或 WebP 格式");
    }

    #[test]
    fn rejects_cover_dimensions_over_limits() {
        assert!(validate_cover_dimensions(8192, 1).is_ok());
        assert!(validate_cover_dimensions(8193, 1).is_err());
        assert!(validate_cover_dimensions(8000, 6000).is_err());
    }
}
