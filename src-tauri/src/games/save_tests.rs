use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::{models::Game, persist_with_cover_rollback, repository};

#[test]
fn removes_the_new_cover_when_database_insert_fails() {
    let test_directory = std::env::temp_dir().join(format!(
        "game-shift-cover-insert-rollback-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&test_directory).unwrap();
    let pending_cover_path = test_directory.join("cover-remote-new.jpg");
    fs::write(&pending_cover_path, b"new cover").unwrap();
    let pending_cover = pending_cover_path.to_string_lossy().into_owned();

    let mut connection = Connection::open_in_memory().unwrap();
    create_games_table(&connection);
    connection
        .execute_batch(
            "
            CREATE TRIGGER fail_game_insert
            BEFORE INSERT ON games
            BEGIN
                SELECT RAISE(FAIL, 'forced insert failure');
            END;
            ",
        )
        .unwrap();
    let game = sample_game(pending_cover.clone());

    let result = persist_with_cover_rollback(game.cover.as_deref(), None, || {
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        repository::insert(&transaction, &game)?;
        let persisted = repository::get_by_id(&transaction, &game.id)?
            .ok_or_else(|| "游戏创建后无法读取".to_string())?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(persisted)
    });

    assert!(result.unwrap_err().contains("forced insert failure"));
    assert!(repository::get_by_id(&connection, &game.id)
        .unwrap()
        .is_none());
    assert!(!pending_cover_path.exists());

    drop(connection);
    fs::remove_dir_all(test_directory).unwrap();
}

#[test]
fn rolls_back_the_new_cover_when_database_update_fails() {
    let test_directory = std::env::temp_dir().join(format!(
        "game-shift-cover-rollback-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&test_directory).unwrap();
    let old_cover_path = test_directory.join("cover-manual-old.jpg");
    let pending_cover_path = test_directory.join("cover-remote-new.jpg");
    fs::write(&old_cover_path, b"old cover").unwrap();
    fs::write(&pending_cover_path, b"new cover").unwrap();
    let old_cover = old_cover_path.to_string_lossy().into_owned();
    let pending_cover = pending_cover_path.to_string_lossy().into_owned();

    let mut connection = Connection::open_in_memory().unwrap();
    create_games_table(&connection);
    let existing = sample_game(old_cover.clone());
    repository::insert(&connection, &existing).unwrap();
    connection
        .execute_batch(
            "
            CREATE TRIGGER fail_game_update
            BEFORE UPDATE ON games
            BEGIN
                SELECT RAISE(FAIL, 'forced update failure');
            END;
            ",
        )
        .unwrap();

    let mut updated = existing.clone();
    updated.name = "Updated name".to_string();
    updated.cover = Some(pending_cover.clone());
    let result =
        persist_with_cover_rollback(updated.cover.as_deref(), existing.cover.as_deref(), || {
            let transaction = connection
                .transaction()
                .map_err(|error| error.to_string())?;
            repository::update_metadata(&transaction, &updated)?;
            let persisted = repository::get_by_id(&transaction, &updated.id)?
                .ok_or_else(|| "游戏更新后无法读取".to_string())?;
            transaction.commit().map_err(|error| error.to_string())?;
            Ok(persisted)
        });

    assert!(result.unwrap_err().contains("forced update failure"));
    let stored = repository::get_by_id(&connection, &existing.id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.name, existing.name);
    assert_eq!(stored.cover.as_deref(), Some(old_cover.as_str()));
    assert!(old_cover_path.is_file());
    assert!(!pending_cover_path.exists());

    drop(connection);
    fs::remove_dir_all(test_directory).unwrap();
}

fn create_games_table(connection: &Connection) {
    connection
        .execute_batch(
            "
            CREATE TABLE games (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                exe_path TEXT NOT NULL UNIQUE,
                folder_path TEXT NOT NULL,
                icon TEXT,
                cover TEXT,
                args TEXT,
                work_dir TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                favorite_time INTEGER,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_play_time INTEGER,
                create_time INTEGER NOT NULL,
                update_time INTEGER NOT NULL
            );
            ",
        )
        .unwrap();
}

fn sample_game(cover: String) -> Game {
    Game {
        id: "game-test".to_string(),
        name: "Original name".to_string(),
        exe_path: r"C:\Games\Example\game.exe".to_string(),
        folder_path: r"C:\Games\Example".to_string(),
        icon: None,
        cover: Some(cover),
        args: None,
        work_dir: Some(r"C:\Games\Example".to_string()),
        favorite: false,
        favorite_time: None,
        play_count: 0,
        last_play_time: None,
        create_time: 1,
        update_time: 1,
    }
}
