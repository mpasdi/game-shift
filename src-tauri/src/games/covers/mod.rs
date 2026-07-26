//! 联网封面模块。
//!
//! `models` 定义 Game Shift 内部统一的数据格式，`provider` 定义第三方封面服务
//! 必须实现的能力。以后接入或更换数据源时，游戏保存和前端展示无需跟着改写。

pub(crate) mod commands;
pub(crate) mod download;
pub(crate) mod models;
pub(crate) mod provider;
pub(crate) mod steamgriddb;

use tauri::AppHandle;

use self::models::{CoverCandidate, CoverSearchResult};
use self::provider::CoverProvider;
use crate::settings;

async fn search_cover_candidates(
    app: &AppHandle,
    query: &str,
) -> Result<CoverSearchResult, String> {
    let provider = settings::online_cover_provider(app)?;
    let query = normalize_search_query(query)?;
    let mut games = provider
        .search_games(query)
        .await
        .map_err(|error| settings::record_online_cover_provider_error(app, error))?;
    let matched_game = games.first().cloned();
    let alternative_games = if games.len() > 1 {
        games.drain(1..).collect()
    } else {
        Vec::new()
    };
    let candidates = match matched_game.as_ref() {
        Some(game) => provider
            .search_covers(&game.provider_game_id)
            .await
            .map_err(|error| settings::record_online_cover_provider_error(app, error))?,
        None => Vec::new(),
    };

    Ok(CoverSearchResult {
        matched_game,
        alternative_games,
        candidates,
    })
}

async fn list_cover_candidates(
    app: &AppHandle,
    provider_id: &str,
    provider_game_id: &str,
) -> Result<Vec<CoverCandidate>, String> {
    let provider = settings::online_cover_provider(app)?;
    if provider_id.trim() != provider.provider_id() {
        return Err("不支持的联网封面数据源".to_string());
    }
    provider
        .search_covers(provider_game_id.trim())
        .await
        .map_err(|error| settings::record_online_cover_provider_error(app, error))
}

fn normalize_search_query(query: &str) -> Result<&str, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("游戏名称不能为空".to_string());
    }
    if query.chars().count() > 120 || query.chars().any(char::is_control) {
        return Err("游戏名称搜索词格式无效".to_string());
    }
    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::normalize_search_query;

    #[test]
    fn validates_cover_search_queries() {
        assert_eq!(
            normalize_search_query("  Elden Ring  ").unwrap(),
            "Elden Ring"
        );
        assert!(normalize_search_query("   ").is_err());
        assert!(normalize_search_query("game\nname").is_err());
        assert!(normalize_search_query(&"x".repeat(121)).is_err());
    }
}
