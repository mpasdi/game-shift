use rusqlite::{params, Connection, OptionalExtension};

use super::models::{StoredApiKeyStatus, StoredOnlineCoverSettings};

const ONLINE_COVERS_ENABLED_KEY: &str = "online_covers_enabled";
const STEAMGRIDDB_API_KEY: &str = "steamgriddb_api_key";
const STEAMGRIDDB_API_KEY_STATUS: &str = "steamgriddb_api_key_status";

pub(super) fn get(connection: &Connection) -> Result<StoredOnlineCoverSettings, String> {
    let enabled =
        get_value(connection, ONLINE_COVERS_ENABLED_KEY)?.is_some_and(|value| value == "true");
    let api_key =
        get_value(connection, STEAMGRIDDB_API_KEY)?.filter(|value| !value.trim().is_empty());
    let api_key_status = match get_value(connection, STEAMGRIDDB_API_KEY_STATUS)?.as_deref() {
        Some("valid") => StoredApiKeyStatus::Valid,
        Some("invalid") => StoredApiKeyStatus::Invalid,
        _ => StoredApiKeyStatus::Unknown,
    };

    Ok(StoredOnlineCoverSettings {
        enabled,
        api_key,
        api_key_status,
    })
}

pub(super) fn set_enabled(connection: &Connection, enabled: bool, now: i64) -> Result<(), String> {
    put_value(
        connection,
        ONLINE_COVERS_ENABLED_KEY,
        if enabled { "true" } else { "false" },
        now,
    )
}

pub(super) fn save_api_key(
    connection: &mut Connection,
    api_key: &str,
    now: i64,
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    put_value(&transaction, STEAMGRIDDB_API_KEY, api_key, now)?;
    put_value(&transaction, STEAMGRIDDB_API_KEY_STATUS, "valid", now)?;
    transaction.commit().map_err(|error| error.to_string())
}

pub(super) fn delete_api_key(connection: &mut Connection) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM settings WHERE key IN (?1, ?2)",
            params![STEAMGRIDDB_API_KEY, STEAMGRIDDB_API_KEY_STATUS],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub(super) fn set_api_key_status(
    connection: &Connection,
    status: StoredApiKeyStatus,
    now: i64,
) -> Result<(), String> {
    let value = match status {
        StoredApiKeyStatus::Unknown => "unknown",
        StoredApiKeyStatus::Valid => "valid",
        StoredApiKeyStatus::Invalid => "invalid",
    };
    put_value(connection, STEAMGRIDDB_API_KEY_STATUS, value, now)
}

fn get_value(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    connection
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|error| error.to_string())
}

fn put_value(connection: &Connection, key: &str, value: &str, now: i64) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO settings (key, value, update_time)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, update_time = excluded.update_time",
            params![key, value, now],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{delete_api_key, get, save_api_key, set_api_key_status, set_enabled};
    use crate::settings::models::StoredApiKeyStatus;

    fn connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    update_time INTEGER NOT NULL
                );",
            )
            .unwrap();
        connection
    }

    #[test]
    fn defaults_to_disabled_without_a_key() {
        let settings = get(&connection()).unwrap();
        assert!(!settings.enabled);
        assert_eq!(settings.api_key, None);
        assert_eq!(settings.api_key_status, StoredApiKeyStatus::Unknown);
    }

    #[test]
    fn persists_switch_key_and_validation_status_independently() {
        let mut connection = connection();
        set_enabled(&connection, true, 1).unwrap();
        save_api_key(&mut connection, "secret-key-1234", 2).unwrap();
        assert_eq!(
            get(&connection).unwrap().api_key_status,
            StoredApiKeyStatus::Valid
        );
        set_api_key_status(&connection, StoredApiKeyStatus::Invalid, 3).unwrap();

        let settings = get(&connection).unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.api_key.as_deref(), Some("secret-key-1234"));
        assert_eq!(settings.api_key_status, StoredApiKeyStatus::Invalid);

        set_enabled(&connection, false, 4).unwrap();
        let disabled = get(&connection).unwrap();
        assert!(!disabled.enabled);
        assert!(disabled.api_key.is_some());
    }

    #[test]
    fn deleting_the_key_preserves_the_switch() {
        let mut connection = connection();
        set_enabled(&connection, true, 1).unwrap();
        save_api_key(&mut connection, "secret-key", 2).unwrap();
        delete_api_key(&mut connection).unwrap();

        let settings = get(&connection).unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.api_key, None);
        assert_eq!(settings.api_key_status, StoredApiKeyStatus::Unknown);
    }
}
