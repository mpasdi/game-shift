//! 联网封面的领域模型。
//!
//! 这里的类型属于 Game Shift，不直接暴露 SteamGridDB 等第三方 API 的原始结构。
//! Provider 负责把外部响应转换为这些稳定类型，再交给 command 和前端使用。

/// 第三方数据源中与搜索词匹配的一个游戏。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameMatch {
    /// 数据源标识，例如 `steamgriddb`。
    pub(crate) provider: String,
    /// 游戏在第三方数据源中的 ID，不是本地 `games` 表的游戏 ID。
    pub(crate) provider_game_id: String,
    /// 第三方数据源提供的游戏名称。
    pub(crate) name: String,
    /// 用于区分重名游戏；数据源没有提供时为空。
    pub(crate) release_year: Option<i32>,
}

/// 某个匹配游戏下可供用户选择的一张封面。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CoverCandidate {
    /// 产生该候选的封面数据源。
    pub(crate) provider: String,
    /// 封面在数据源中的资源 ID，保存时后端使用它重新定位可信资源。
    pub(crate) asset_id: String,
    /// 该封面所属游戏在数据源中的 ID。
    pub(crate) provider_game_id: String,
    /// 仅用于候选界面预览；保存请求不会把这个 URL 当作下载依据。
    pub(crate) preview_url: String,
    /// 原始候选图片宽度，用于展示尺寸和筛选比例。
    pub(crate) width: u32,
    /// 原始候选图片高度，用于展示尺寸和筛选比例。
    pub(crate) height: u32,
}

/// 一次封面搜索返回给前端的完整结果。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CoverSearchResult {
    /// 后端自动判断的最佳游戏匹配；没有匹配时为空。
    pub(crate) matched_game: Option<GameMatch>,
    /// 自动匹配不正确时，供用户切换的其他游戏结果。
    pub(crate) alternative_games: Vec<GameMatch>,
    /// 当前最佳匹配游戏下的封面候选。
    pub(crate) candidates: Vec<CoverCandidate>,
}

/// 用户保存游戏时对封面的明确处理意图。
///
/// 使用枚举可以避免把“没有修改”“选择本地图片”“选择网络图片”和“主动删除”
/// 都塞进一个可空路径，导致后端无法判断用户真正想做什么。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum CoverSelection {
    /// 保留现有封面，不执行任何封面操作。
    Unchanged,
    /// 使用用户从本机选择的图片。
    Local { path: String },
    /// 使用联网候选。只传数据源和资源 ID，不接受任意下载 URL。
    Remote {
        provider: String,
        #[serde(rename = "assetId")]
        asset_id: String,
    },
    /// 主动移除当前封面。
    Remove,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CoverCandidate, CoverSearchResult, CoverSelection, GameMatch};

    #[test]
    fn serializes_remote_cover_selection_without_a_download_url() {
        let selection = CoverSelection::Remote {
            provider: "steamgriddb".to_string(),
            asset_id: "grid-42".to_string(),
        };

        assert_eq!(
            serde_json::to_value(selection).unwrap(),
            json!({
                "type": "remote",
                "provider": "steamgriddb",
                "assetId": "grid-42"
            })
        );
    }

    #[test]
    fn serializes_local_cover_selection() {
        let selection = CoverSelection::Local {
            path: r"C:\Covers\game.png".to_string(),
        };

        assert_eq!(
            serde_json::to_value(selection).unwrap(),
            json!({
                "type": "local",
                "path": r"C:\Covers\game.png"
            })
        );
    }

    #[test]
    fn serializes_cover_search_result_with_camel_case_fields() {
        let game = GameMatch {
            provider: "steamgriddb".to_string(),
            provider_game_id: "game-7".to_string(),
            name: "Example Game".to_string(),
            release_year: Some(2026),
        };
        let result = CoverSearchResult {
            matched_game: Some(game),
            alternative_games: Vec::new(),
            candidates: vec![CoverCandidate {
                provider: "steamgriddb".to_string(),
                asset_id: "grid-42".to_string(),
                provider_game_id: "game-7".to_string(),
                preview_url: "https://example.invalid/grid-42.jpg".to_string(),
                width: 600,
                height: 900,
            }],
        };

        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["matchedGame"]["providerGameId"], "game-7");
        assert_eq!(
            value["candidates"][0]["previewUrl"],
            "https://example.invalid/grid-42.jpg"
        );
    }

    #[test]
    fn serializes_cover_selection_states() {
        assert_eq!(
            serde_json::to_value(CoverSelection::Unchanged).unwrap(),
            json!({ "type": "unchanged" })
        );
        assert_eq!(
            serde_json::to_value(CoverSelection::Remove).unwrap(),
            json!({ "type": "remove" })
        );
    }
}
