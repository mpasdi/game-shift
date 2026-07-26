#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) exe_path: String,
    pub(super) folder_path: String,
    pub(super) icon: Option<String>,
    pub(super) cover: Option<String>,
    pub(super) args: Option<String>,
    pub(super) work_dir: Option<String>,
    pub(super) favorite: bool,
    pub(super) favorite_time: Option<i64>,
    pub(super) play_count: i64,
    pub(super) last_play_time: Option<i64>,
    pub(super) create_time: i64,
    pub(super) update_time: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGamePayload {
    pub(super) name: String,
    pub(super) exe_path: String,
    pub(super) work_dir: Option<String>,
    pub(super) args: Option<String>,
    pub(super) cover_path: Option<String>,
    pub(super) cover_selection: Option<CoverSelection>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGamePayload {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) exe_path: String,
    pub(super) work_dir: Option<String>,
    pub(super) args: Option<String>,
    pub(super) cover_path: Option<String>,
    pub(super) cover_selection: Option<CoverSelection>,
    pub(super) favorite: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCandidate {
    pub(super) name: String,
    pub(super) exe_path: String,
    pub(super) folder_path: String,
    pub(super) exe_file_name: String,
    pub(super) exists: bool,
}

pub(super) struct NormalizedGameFields {
    pub(super) name: String,
    pub(super) exe_path: String,
    pub(super) folder_path: String,
    pub(super) work_dir: Option<String>,
    pub(super) args: Option<String>,
}
use super::covers::models::CoverSelection;
