use tauri::AppHandle;

use super::models::{CreateGamePayload, Game, ScanCandidate, UpdateGamePayload};

#[tauri::command]
pub fn list_games_command(app: AppHandle) -> Result<Vec<Game>, String> {
    super::list_games(&app)
}

#[tauri::command]
pub fn get_game_command(app: AppHandle, id: String) -> Result<Option<Game>, String> {
    super::get_game(&app, &id)
}

#[tauri::command]
pub async fn create_game_command(
    app: AppHandle,
    payload: CreateGamePayload,
) -> Result<Game, String> {
    super::create_game(&app, payload).await
}

#[tauri::command]
pub async fn update_game_command(
    app: AppHandle,
    payload: UpdateGamePayload,
) -> Result<Game, String> {
    super::update_game(&app, payload).await
}

#[tauri::command]
pub fn delete_game_command(app: AppHandle, id: String) -> Result<(), String> {
    super::delete_game(&app, &id)
}

#[tauri::command]
pub fn launch_game_command(app: AppHandle, id: String) -> Result<Game, String> {
    super::launcher::launch_game(&app, &id)
}

#[tauri::command]
pub async fn scan_games_command(
    app: AppHandle,
    directory: String,
) -> Result<Vec<ScanCandidate>, String> {
    tauri::async_runtime::spawn_blocking(move || super::scanner::scan_games(&app, &directory))
        .await
        .map_err(|error| error.to_string())?
}
