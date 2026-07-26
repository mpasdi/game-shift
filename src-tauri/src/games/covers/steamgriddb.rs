//! SteamGridDB 联网封面数据源。
//!
//! API Key 只用于 Rust 发出的鉴权请求，不进入 URL，也不会返回给前端。

use std::fmt;
use std::time::Duration;

use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;

use super::download::validate_trusted_url;
use super::models::{CoverCandidate, GameMatch, ResolvedCoverAsset};
use super::provider::{CoverProvider, ProviderFuture};

const PROVIDER_ID: &str = "steamgriddb";
const API_BASE_URL: &str = "https://www.steamgriddb.com/api/v2/";
const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const VERTICAL_GRID_DIMENSIONS: &str = "600x900,342x482,660x930";
const AUTH_TEST_GAME_ID: &str = "1";

#[derive(Debug)]
pub(crate) enum SteamGridDbError {
    Unauthorized,
    Message(String),
}

impl fmt::Display for SteamGridDbError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => formatter.write_str("无效 SteamGridDB Key"),
            Self::Message(message) => formatter.write_str(message),
        }
    }
}

pub(crate) struct SteamGridDbProvider {
    api_key: String,
    client: reqwest::Client,
    base_url: Url,
}

impl SteamGridDbProvider {
    pub(crate) fn new(api_key: impl Into<String>) -> Result<Self, String> {
        let api_key = api_key.into().trim().to_string();
        if api_key.is_empty() {
            return Err("SteamGridDB API Key 不能为空".to_string());
        }

        // reqwest 0.13 默认使用 aws-lc；本地桌面工具改用更轻的 ring 后端。
        // 若进程中已有其他 rustls Provider，保留现有选择即可。
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .https_only(true)
            .user_agent(format!("Game Shift/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| "无法初始化联网封面客户端".to_string())?;
        let base_url =
            Url::parse(API_BASE_URL).map_err(|_| "SteamGridDB API 地址配置无效".to_string())?;

        Ok(Self {
            api_key,
            client,
            base_url,
        })
    }

    fn endpoint(&self, prefix: &str, final_segment: &str) -> Result<Url, String> {
        let mut url = self
            .base_url
            .join(prefix)
            .map_err(|_| "无法构造 SteamGridDB 请求地址".to_string())?;
        url.path_segments_mut()
            .map_err(|_| "无法构造 SteamGridDB 请求地址".to_string())?
            .pop_if_empty()
            .push(final_segment);
        Ok(url)
    }

    async fn get_data<T: DeserializeOwned>(&self, url: Url) -> Result<Option<T>, SteamGridDbError> {
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|error| SteamGridDbError::Message(map_request_error(error)))?;

        match response.status() {
            StatusCode::UNAUTHORIZED => return Err(SteamGridDbError::Unauthorized),
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(SteamGridDbError::Message(
                    "SteamGridDB 请求过于频繁，请稍后再试".to_string(),
                ))
            }
            StatusCode::NOT_FOUND => return Ok(None),
            status if !status.is_success() => {
                return Err(SteamGridDbError::Message(format!(
                    "SteamGridDB 请求失败（HTTP {}）",
                    status.as_u16()
                )))
            }
            _ => {}
        }

        let body = response
            .bytes()
            .await
            .map_err(|_| SteamGridDbError::Message("读取 SteamGridDB 响应失败".to_string()))?;
        decode_body(&body)
            .map(Some)
            .map_err(SteamGridDbError::Message)
    }

    async fn fetch_grids(&self, provider_game_id: &str) -> Result<Vec<SteamGridDbGrid>, String> {
        let game_id = parse_numeric_id(provider_game_id, "游戏")?;
        let mut url = self.endpoint("grids/game/", &game_id.to_string())?;
        url.query_pairs_mut()
            .append_pair("dimensions", VERTICAL_GRID_DIMENSIONS)
            .append_pair("types", "static")
            .append_pair("nsfw", "false")
            .append_pair("humor", "false")
            .append_pair("epilepsy", "false")
            .append_pair("limit", "50");

        Ok(self
            .get_data(url)
            .await
            .map_err(|error| error.to_string())?
            .unwrap_or_default())
    }

    /// 使用严格鉴权的封面端点验证 API Key，不接触本地游戏数据。
    /// 资源不存在时会在鉴权通过后返回 404；无效或缺失的 Key 返回 401。
    pub(crate) async fn test_connection(&self) -> Result<(), SteamGridDbError> {
        let url = self
            .endpoint("grids/game/", AUTH_TEST_GAME_ID)
            .map_err(SteamGridDbError::Message)?;
        let _: Option<Vec<SteamGridDbGrid>> = self.get_data(url).await?;
        Ok(())
    }
}

impl CoverProvider for SteamGridDbProvider {
    fn provider_id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn search_games<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<GameMatch>> {
        Box::pin(async move {
            let query = query.trim();
            if query.is_empty() {
                return Err("游戏名称不能为空".to_string());
            }

            let url = self.endpoint("search/autocomplete/", query)?;
            let games: Vec<SteamGridDbGame> = self
                .get_data(url)
                .await
                .map_err(|error| error.to_string())?
                .unwrap_or_default();
            Ok(games.into_iter().map(GameMatch::from).collect())
        })
    }

    fn search_covers<'a>(
        &'a self,
        provider_game_id: &'a str,
    ) -> ProviderFuture<'a, Vec<CoverCandidate>> {
        Box::pin(async move {
            let grids = self.fetch_grids(provider_game_id).await?;
            Ok(grids
                .into_iter()
                .filter_map(|grid| grid.into_candidate(provider_game_id))
                .collect())
        })
    }

    fn resolve_cover<'a>(
        &'a self,
        provider_game_id: &'a str,
        asset_id: &'a str,
    ) -> ProviderFuture<'a, Option<ResolvedCoverAsset>> {
        Box::pin(async move {
            let asset_id = parse_numeric_id(asset_id, "封面")?;
            let grids = self.fetch_grids(provider_game_id).await?;

            Ok(grids
                .into_iter()
                .find(|grid| grid.id == asset_id)
                .map(|grid| ResolvedCoverAsset {
                    provider: PROVIDER_ID.to_string(),
                    asset_id: grid.id.to_string(),
                    provider_game_id: provider_game_id.to_string(),
                    download_url: grid.url,
                }))
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct ApiEnvelope<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, serde::Deserialize)]
struct SteamGridDbGame {
    id: u64,
    name: String,
}

impl From<SteamGridDbGame> for GameMatch {
    fn from(game: SteamGridDbGame) -> Self {
        Self {
            provider: PROVIDER_ID.to_string(),
            provider_game_id: game.id.to_string(),
            name: game.name,
            release_year: None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct SteamGridDbGrid {
    id: u64,
    url: String,
    thumb: String,
}

impl SteamGridDbGrid {
    fn into_candidate(self, provider_game_id: &str) -> Option<CoverCandidate> {
        let preview_url = Url::parse(&self.thumb).ok()?;
        validate_trusted_url(&preview_url).ok()?;
        Some(CoverCandidate {
            provider: PROVIDER_ID.to_string(),
            asset_id: self.id.to_string(),
            provider_game_id: provider_game_id.to_string(),
            preview_url: self.thumb,
            width: None,
            height: None,
        })
    }
}

fn decode_body<T: DeserializeOwned>(body: &[u8]) -> Result<T, String> {
    let envelope: ApiEnvelope<T> =
        serde_json::from_slice(body).map_err(|_| "SteamGridDB 返回了无法识别的数据".to_string())?;
    if !envelope.success {
        return Err("SteamGridDB 未能完成请求".to_string());
    }
    envelope
        .data
        .ok_or_else(|| "SteamGridDB 响应缺少数据".to_string())
}

fn parse_numeric_id(value: &str, kind: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("SteamGridDB {kind} ID 无效"))
}

fn map_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "连接 SteamGridDB 超时，请稍后再试".to_string()
    } else if error.is_connect() {
        "无法连接 SteamGridDB，请检查网络后重试".to_string()
    } else {
        "请求 SteamGridDB 失败，请稍后再试".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decode_body, parse_numeric_id, SteamGridDbGame, SteamGridDbGrid, SteamGridDbProvider,
    };
    use crate::games::covers::provider::CoverProvider;

    #[test]
    fn rejects_an_empty_api_key() {
        assert!(SteamGridDbProvider::new("   ").is_err());
    }

    #[test]
    fn decodes_search_results_into_domain_games() {
        let body = br#"{
            "success": true,
            "data": [
                { "id": 123, "name": "Example Game", "types": ["game"], "verified": true }
            ]
        }"#;
        let games: Vec<SteamGridDbGame> = decode_body(body).unwrap();
        let game = super::GameMatch::from(games.into_iter().next().unwrap());

        assert_eq!(game.provider, "steamgriddb");
        assert_eq!(game.provider_game_id, "123");
        assert_eq!(game.name, "Example Game");
        assert_eq!(game.release_year, None);
    }

    #[test]
    fn maps_grid_thumbnail_without_exposing_download_url() {
        let body = br#"{
            "success": true,
            "page": 0,
            "total": 1,
            "limit": 50,
            "data": [
                {
                    "id": 456,
                    "score": 5,
                    "style": "alternate",
                    "url": "https://cdn2.steamgriddb.com/full.jpg",
                    "thumb": "https://cdn2.steamgriddb.com/thumb.jpg",
                    "tags": [],
                    "author": { "name": "Author", "steam64": "1", "avatar": "https://example.invalid/a.jpg" }
                }
            ]
        }"#;
        let grids: Vec<SteamGridDbGrid> = decode_body(body).unwrap();
        let candidate = grids
            .into_iter()
            .next()
            .unwrap()
            .into_candidate("123")
            .unwrap();

        assert_eq!(candidate.asset_id, "456");
        assert_eq!(candidate.provider_game_id, "123");
        assert_eq!(
            candidate.preview_url,
            "https://cdn2.steamgriddb.com/thumb.jpg"
        );
        assert_eq!(candidate.width, None);
        assert_eq!(candidate.height, None);
    }

    #[test]
    fn rejects_failed_or_incomplete_envelopes() {
        assert!(decode_body::<Vec<SteamGridDbGame>>(br#"{"success":false,"data":[]}"#).is_err());
        assert!(decode_body::<Vec<SteamGridDbGame>>(br#"{"success":true}"#).is_err());
    }

    #[test]
    fn validates_provider_ids_before_building_requests() {
        assert_eq!(parse_numeric_id("42", "游戏").unwrap(), 42);
        assert!(parse_numeric_id("../../42", "游戏").is_err());
    }

    #[test]
    fn provider_id_is_stable() {
        let provider = SteamGridDbProvider::new("test-key").unwrap();
        assert_eq!(provider.provider_id(), "steamgriddb");
    }

    #[test]
    fn encodes_search_terms_without_putting_the_key_in_the_url() {
        let provider = SteamGridDbProvider::new("secret-test-key").unwrap();
        let term = "NieR: Automata / GOTY?";
        let url = provider.endpoint("search/autocomplete/", term).unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(
            url.path_segments().unwrap().next_back(),
            Some("NieR:%20Automata%20%2F%20GOTY%3F")
        );
        assert!(!url.as_str().contains("secret-test-key"));
    }

    #[test]
    fn connection_test_uses_an_authenticated_cover_endpoint() {
        let provider = SteamGridDbProvider::new("secret-test-key").unwrap();
        let url = provider
            .endpoint("grids/game/", super::AUTH_TEST_GAME_ID)
            .unwrap();

        assert_eq!(url.path(), "/api/v2/grids/game/1");
        assert!(!url.as_str().contains("secret-test-key"));
    }
}
