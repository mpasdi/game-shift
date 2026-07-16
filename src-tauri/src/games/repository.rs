use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use super::models::Game;
use super::strip_windows_extended_path_prefix;

pub(super) fn list(connection: &Connection) -> Result<Vec<Game>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
                id, name, exe_path, folder_path, icon, cover, args, work_dir,
                favorite, favorite_time, play_count, last_play_time, create_time, update_time
            FROM games
            ORDER BY create_time DESC
            ",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], map_game_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub(super) fn get_by_id(connection: &Connection, id: &str) -> Result<Option<Game>, String> {
    connection
        .query_row(
            "
            SELECT
                id, name, exe_path, folder_path, icon, cover, args, work_dir,
                favorite, favorite_time, play_count, last_play_time, create_time, update_time
            FROM games
            WHERE id = ?1
            ",
            params![id],
            map_game_row,
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub(super) fn insert(connection: &Connection, game: &Game) -> Result<(), String> {
    connection
        .execute(
            "
            INSERT INTO games (
                id, name, exe_path, folder_path, icon, cover, args, work_dir,
                favorite, favorite_time, play_count, last_play_time, create_time, update_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ",
            params![
                game.id,
                game.name,
                game.exe_path,
                game.folder_path,
                game.icon,
                game.cover,
                game.args,
                game.work_dir,
                i64::from(game.favorite),
                game.favorite_time,
                game.play_count,
                game.last_play_time,
                game.create_time,
                game.update_time
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn update_metadata(connection: &Connection, game: &Game) -> Result<(), String> {
    connection
        .execute(
            "
            UPDATE games
            SET name = ?1,
                exe_path = ?2,
                folder_path = ?3,
                icon = ?4,
                cover = ?5,
                args = ?6,
                work_dir = ?7,
                favorite = ?8,
                favorite_time = ?9,
                update_time = ?10
            WHERE id = ?11
            ",
            params![
                game.name,
                game.exe_path,
                game.folder_path,
                game.icon,
                game.cover,
                game.args,
                game.work_dir,
                i64::from(game.favorite),
                game.favorite_time,
                game.update_time,
                game.id
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn update_visual_assets(connection: &Connection, game: &Game) -> Result<(), String> {
    connection
        .execute(
            "
            UPDATE games
            SET icon = ?1,
                cover = ?2,
                update_time = ?3
            WHERE id = ?4
            ",
            params![game.icon, game.cover, game.update_time, game.id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn delete(connection: &Connection, id: &str) -> Result<bool, String> {
    connection
        .execute("DELETE FROM games WHERE id = ?1", params![id])
        .map(|affected| affected > 0)
        .map_err(|error| error.to_string())
}

pub(super) fn record_launch(connection: &Connection, id: &str, now: i64) -> Result<(), String> {
    connection
        .execute(
            "
            UPDATE games
            SET play_count = play_count + 1,
                last_play_time = ?1,
                update_time = ?2
            WHERE id = ?3
            ",
            params![now, now, id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn exe_path_exists(connection: &Connection, exe_path: &str) -> Result<bool, String> {
    count_by_path(connection, exe_path, None).map(|count| count > 0)
}

pub(super) fn exe_path_exists_for_other_game(
    connection: &Connection,
    exe_path: &str,
    id: &str,
) -> Result<bool, String> {
    count_by_path(connection, exe_path, Some(id)).map(|count| count > 0)
}

fn count_by_path(
    connection: &Connection,
    exe_path: &str,
    excluded_id: Option<&str>,
) -> Result<i64, String> {
    let result = match excluded_id {
        Some(id) => connection.query_row(
            "SELECT COUNT(1) FROM games WHERE exe_path = ?1 AND id <> ?2",
            params![exe_path, id],
            |row| row.get(0),
        ),
        None => connection.query_row(
            "SELECT COUNT(1) FROM games WHERE exe_path = ?1",
            params![exe_path],
            |row| row.get(0),
        ),
    };
    result.map_err(|error| error.to_string())
}

pub(super) fn query_existing_exe_paths(connection: &Connection) -> Result<HashSet<String>, String> {
    let mut statement = connection
        .prepare("SELECT exe_path FROM games")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;

    rows.map(|row| row.map(strip_windows_extended_path_prefix))
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())
}

fn map_game_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        name: row.get(1)?,
        exe_path: strip_windows_extended_path_prefix(row.get(2)?),
        folder_path: strip_windows_extended_path_prefix(row.get(3)?),
        icon: row.get(4)?,
        cover: row.get(5)?,
        args: row.get(6)?,
        work_dir: row
            .get::<_, Option<String>>(7)?
            .map(strip_windows_extended_path_prefix),
        favorite: row.get::<_, i64>(8)? != 0,
        favorite_time: row.get(9)?,
        play_count: row.get(10)?,
        last_play_time: row.get(11)?,
        create_time: row.get(12)?,
        update_time: row.get(13)?,
    })
}
