use std::fs::{self, File};
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{ColorType, DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader};
use tauri::{AppHandle, Manager};

use super::{current_timestamp_millis, path_to_string};

const MAX_COVER_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_COVER_DIMENSION: u32 = 8192;
const MAX_COVER_PIXELS: u64 = 40_000_000;
const MAX_CACHED_COVER_DIMENSION: u32 = 2400;
const COVER_JPEG_QUALITY: u8 = 88;
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

pub(super) fn cache_manual_cover(
    app: &AppHandle,
    source_path: &str,
    game_id: &str,
) -> Result<String, String> {
    let source = PathBuf::from(source_path.trim());
    if !source.is_file() {
        return Err("选择的封面文件不存在".to_string());
    }

    let timestamp = current_timestamp_millis()?;
    cache_cover_image(app, &source, game_id, &format!("cover-manual-{timestamp}"))
}

pub(super) fn cache_remote_cover(
    app: &AppHandle,
    bytes: Vec<u8>,
    game_id: &str,
) -> Result<String, String> {
    if bytes.len() > MAX_COVER_FILE_SIZE_BYTES as usize {
        return Err("联网封面文件不能超过 10 MB".to_string());
    }
    let image = decode_cover_image(bytes)?;
    let timestamp = current_timestamp_millis()?;
    encode_cached_cover(app, game_id, &format!("cover-remote-{timestamp}"), image)
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

pub(super) fn decode_cover_image(bytes: Vec<u8>) -> Result<DynamicImage, String> {
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

pub(super) fn validate_cover_dimensions(width: u32, height: u32) -> Result<(), String> {
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

pub(super) fn cleanup_stale_cover_files(
    app: &AppHandle,
    game_id: &str,
    current_cover: Option<&str>,
) {
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
                    || value.starts_with("cover-remote-")
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

pub(super) fn detect_and_cache_cover(
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

pub(super) fn should_refresh_icon(icon: Option<&str>) -> bool {
    match icon {
        None => true,
        Some(path) => Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ico")),
    }
}
pub(super) fn extract_game_icon(
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat};

    use super::{decode_cover_image, validate_cover_dimensions};

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
