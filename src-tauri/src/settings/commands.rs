use tauri::AppHandle;

use super::models::{AppUpdateSettings, OnlineCoverSettings};

#[tauri::command]
pub(crate) fn get_app_update_settings_command(app: AppHandle) -> Result<AppUpdateSettings, String> {
    super::get_app_update_settings(&app)
}

#[tauri::command]
pub(crate) fn set_auto_check_updates_enabled_command(
    app: AppHandle,
    enabled: bool,
) -> Result<AppUpdateSettings, String> {
    super::set_auto_check_updates_enabled(&app, enabled)
}

#[tauri::command]
pub(crate) fn get_online_cover_settings_command(
    app: AppHandle,
) -> Result<OnlineCoverSettings, String> {
    super::get_online_cover_settings(&app)
}

#[tauri::command]
pub(crate) fn set_online_covers_enabled_command(
    app: AppHandle,
    enabled: bool,
) -> Result<OnlineCoverSettings, String> {
    super::set_online_covers_enabled(&app, enabled)
}

#[tauri::command]
pub(crate) async fn save_steamgriddb_api_key_command(
    app: AppHandle,
    api_key: String,
) -> Result<OnlineCoverSettings, String> {
    super::save_steamgriddb_api_key(&app, api_key).await
}

#[tauri::command]
pub(crate) fn delete_steamgriddb_api_key_command(
    app: AppHandle,
) -> Result<OnlineCoverSettings, String> {
    super::delete_steamgriddb_api_key(&app)
}

#[tauri::command]
pub(crate) async fn test_steamgriddb_connection_command(
    app: AppHandle,
) -> Result<OnlineCoverSettings, String> {
    super::test_steamgriddb_connection(&app).await
}
