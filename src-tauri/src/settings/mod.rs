pub(crate) mod commands;
mod models;
mod repository;

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::AppHandle;

use crate::db;
use crate::games::covers::steamgriddb::{SteamGridDbError, SteamGridDbProvider};

use models::{OnlineCoverSettings, StoredApiKeyStatus};

fn get_online_cover_settings(app: &AppHandle) -> Result<OnlineCoverSettings, String> {
    let connection = db::open_connection(app)?;
    Ok(repository::get(&connection)?.to_public())
}

pub(crate) fn online_cover_provider(app: &AppHandle) -> Result<SteamGridDbProvider, String> {
    let connection = db::open_connection(app)?;
    let settings = repository::get(&connection)?;
    if !settings.enabled {
        return Err("联网封面尚未启用".to_string());
    }
    if settings.api_key_status == StoredApiKeyStatus::Invalid {
        return Err("SteamGridDB API Key 无效，请在设置中重新配置".to_string());
    }
    let api_key = settings
        .api_key
        .ok_or_else(|| "请先在设置中保存 SteamGridDB API Key".to_string())?;
    SteamGridDbProvider::new(api_key)
}

pub(crate) fn record_online_cover_provider_error(app: &AppHandle, error: String) -> String {
    if error == SteamGridDbError::Unauthorized.to_string() {
        if let Ok(connection) = db::open_connection(app) {
            if let Ok(now) = current_timestamp_millis() {
                let _ =
                    repository::set_api_key_status(&connection, StoredApiKeyStatus::Invalid, now);
            }
        }
    }
    error
}

fn set_online_covers_enabled(
    app: &AppHandle,
    enabled: bool,
) -> Result<OnlineCoverSettings, String> {
    let connection = db::open_connection(app)?;
    repository::set_enabled(&connection, enabled, current_timestamp_millis()?)?;
    Ok(repository::get(&connection)?.to_public())
}

async fn save_steamgriddb_api_key(
    app: &AppHandle,
    api_key: String,
) -> Result<OnlineCoverSettings, String> {
    let api_key = normalize_api_key(api_key)?;
    let provider = SteamGridDbProvider::new(api_key.clone())?;
    provider
        .test_connection()
        .await
        .map_err(|error| error.to_string())?;

    let mut connection = db::open_connection(app)?;
    repository::save_api_key(&mut connection, &api_key, current_timestamp_millis()?)?;
    Ok(repository::get(&connection)?.to_public())
}

fn delete_steamgriddb_api_key(app: &AppHandle) -> Result<OnlineCoverSettings, String> {
    let mut connection = db::open_connection(app)?;
    repository::delete_api_key(&mut connection)?;
    Ok(repository::get(&connection)?.to_public())
}

async fn test_steamgriddb_connection(app: &AppHandle) -> Result<OnlineCoverSettings, String> {
    let api_key = {
        let connection = db::open_connection(app)?;
        repository::get(&connection)?
            .api_key
            .ok_or_else(|| "请先保存 SteamGridDB API Key".to_string())?
    };
    let provider = SteamGridDbProvider::new(api_key)?;

    match provider.test_connection().await {
        Ok(()) => {
            let connection = db::open_connection(app)?;
            repository::set_api_key_status(
                &connection,
                StoredApiKeyStatus::Valid,
                current_timestamp_millis()?,
            )?;
            Ok(repository::get(&connection)?.to_public())
        }
        Err(SteamGridDbError::Unauthorized) => {
            let connection = db::open_connection(app)?;
            repository::set_api_key_status(
                &connection,
                StoredApiKeyStatus::Invalid,
                current_timestamp_millis()?,
            )?;
            Err(SteamGridDbError::Unauthorized.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn normalize_api_key(api_key: String) -> Result<String, String> {
    let api_key = api_key.trim().to_string();
    if api_key.is_empty() {
        return Err("SteamGridDB API Key 不能为空".to_string());
    }
    if api_key.len() > 512 || api_key.chars().any(char::is_control) {
        return Err("SteamGridDB API Key 格式无效".to_string());
    }
    Ok(api_key)
}

fn current_timestamp_millis() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    i64::try_from(duration.as_millis()).map_err(|_| "当前时间戳超出范围".to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_api_key;

    #[test]
    fn normalizes_api_keys_without_logging_or_exposing_them() {
        assert_eq!(
            normalize_api_key("  key-1234  ".to_string()).unwrap(),
            "key-1234"
        );
        assert!(normalize_api_key("   ".to_string()).is_err());
        assert!(normalize_api_key("key\nvalue".to_string()).is_err());
    }
}
