use std::collections::HashSet;
use std::fs;
use std::path::Path;

use tauri::AppHandle;

use crate::db;

use super::models::ScanCandidate;
use super::repository;
use super::{has_exe_extension, normalize_existing_directory, path_to_string};

pub(super) fn scan_games(app: &AppHandle, directory: &str) -> Result<Vec<ScanCandidate>, String> {
    let connection = db::open_connection(app)?;
    let root = normalize_existing_directory(directory)?;
    let existing_exe_paths = repository::query_existing_exe_paths(&connection)?;
    let mut candidates = Vec::new();

    scan_directory(&root, &existing_exe_paths, &mut candidates)?;
    candidates.sort_by(|left, right| {
        left.exists
            .cmp(&right.exists)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok(candidates)
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
