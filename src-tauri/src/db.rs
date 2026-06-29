use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

const DATABASE_FILE_NAME: &str = "game-shift.sqlite3";

pub fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    Ok(data_dir.join(DATABASE_FILE_NAME))
}

pub fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    Connection::open(path).map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let connection = open_connection(app)?;
    run_migrations(&connection)
}

fn run_migrations(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                exe_path TEXT NOT NULL UNIQUE,
                folder_path TEXT NOT NULL,
                icon TEXT,
                cover TEXT,
                args TEXT,
                work_dir TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_play_time INTEGER,
                create_time INTEGER NOT NULL,
                update_time INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_games_name ON games(name);
            CREATE INDEX IF NOT EXISTS idx_games_favorite ON games(favorite);
            CREATE INDEX IF NOT EXISTS idx_games_last_play_time ON games(last_play_time);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                update_time INTEGER NOT NULL
            );
            ",
        )
        .map_err(|error| error.to_string())?;

    ensure_column(connection, "games", "cover", "TEXT")
}

fn ensure_column(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_type: &str,
) -> Result<(), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|error| error.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?;

    for column in columns {
        if column.map_err(|error| error.to_string())? == column_name {
            return Ok(());
        }
    }

    connection
        .execute(
            &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_type}"),
            [],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}
