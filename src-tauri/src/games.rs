use rusqlite::Connection;
use tauri::AppHandle;

use crate::db;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
    name: String,
    exe_path: String,
    folder_path: String,
    icon: Option<String>,
    args: Option<String>,
    work_dir: Option<String>,
    favorite: bool,
    play_count: i64,
    last_play_time: Option<i64>,
    create_time: i64,
    update_time: i64,
}

pub fn list_games(app: &AppHandle) -> Result<Vec<Game>, String> {
    let connection = db::open_connection(app)?;
    query_games(&connection)
}

fn query_games(connection: &Connection) -> Result<Vec<Game>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
                id,
                name,
                exe_path,
                folder_path,
                icon,
                args,
                work_dir,
                favorite,
                play_count,
                last_play_time,
                create_time,
                update_time
            FROM games
            ORDER BY favorite DESC,
                     COALESCE(last_play_time, create_time) DESC,
                     create_time DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
                exe_path: row.get(2)?,
                folder_path: row.get(3)?,
                icon: row.get(4)?,
                args: row.get(5)?,
                work_dir: row.get(6)?,
                favorite: row.get::<_, i64>(7)? != 0,
                play_count: row.get(8)?,
                last_play_time: row.get(9)?,
                create_time: row.get(10)?,
                update_time: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_games_command(app: AppHandle) -> Result<Vec<Game>, String> {
    list_games(&app)
}