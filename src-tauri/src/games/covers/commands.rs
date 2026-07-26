use tauri::AppHandle;

use super::models::{CoverCandidate, CoverSearchResult};

#[tauri::command]
pub(crate) async fn search_cover_candidates_command(
    app: AppHandle,
    query: String,
) -> Result<CoverSearchResult, String> {
    super::search_cover_candidates(&app, &query).await
}

#[tauri::command]
pub(crate) async fn list_cover_candidates_command(
    app: AppHandle,
    provider: String,
    provider_game_id: String,
) -> Result<Vec<CoverCandidate>, String> {
    super::list_cover_candidates(&app, &provider, &provider_game_id).await
}
