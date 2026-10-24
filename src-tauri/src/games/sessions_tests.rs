use super::*;

fn database() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    crate::db::run_migrations(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO games (id, name, exe_path, folder_path, create_time, update_time)
         VALUES ('game', 'Example', 'game.exe', '.', 1, 1)",
            [],
        )
        .unwrap();
    connection
}

fn session(id: &str, start: i64, duration: i64, end_reason: Option<EndReason>) -> PlaySession {
    PlaySession {
        id: id.into(),
        game_id: "game".into(),
        started_at: start,
        duration_ms: duration,
        end_reason,
    }
}

// 测试场景：重复保存、重新结算不会重复增加启动次数或累计时长。
#[test]
fn checkpoints_and_finalization_are_idempotent_and_totals_are_derived() {
    let mut connection = database();
    let mut first = session("first", 1_000, 0, None);
    save(&mut connection, &first).unwrap();
    first.duration_ms = 30_000;
    save(&mut connection, &first).unwrap();
    save(&mut connection, &first).unwrap();
    first.duration_ms = 45_000;
    first.end_reason = Some(EndReason::GameExited);
    save(&mut connection, &first).unwrap();
    // 已结束的会话保持不可变，不被迟到的检查点或重复退出覆盖。
    first.duration_ms = 90_000;
    save(&mut connection, &first).unwrap();
    let second = session("second", 100_000, 15_000, Some(EndReason::ManagerExited));
    save(&mut connection, &second).unwrap();
    let counts: (i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), SUM(duration_ms) FROM play_sessions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(counts, (2, 60_000));
    let game = super::super::repository::get_by_id(&connection, "game")
        .unwrap()
        .unwrap();
    assert_eq!(game.play_count, 2);
    assert_eq!(game.last_play_time, Some(100_000));
}

// 测试场景：崩溃重启只保留最后落盘进度，重复恢复不改变已结算记录。
#[test]
fn interruption_recovery_uses_checkpoint_and_preserves_completed_sessions() {
    let mut connection = database();
    save(
        &mut connection,
        &session("done", 1_000, 20_000, Some(EndReason::GameExited)),
    )
    .unwrap();
    save(&mut connection, &session("open", 50_000, 30_000, None)).unwrap();
    recover_interrupted(&connection).unwrap();
    recover_interrupted(&connection).unwrap();
    let recovered: (i64, i64, String) = connection
        .query_row(
            "SELECT duration_ms, ended_at, end_reason FROM play_sessions WHERE id = 'open'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(recovered, (30_000, 80_000, "interrupted".into()));
    assert_eq!(
        connection
            .query_row(
                "SELECT end_reason FROM play_sessions WHERE id = 'done'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "game_exited"
    );
    // 恢复后可再次创建同一游戏的新会话。
    save(&mut connection, &session("new", 1_000_000, 0, None)).unwrap();
}

// 测试场景：启动记录写入失败时事务整体回退，重试后只记录一次。
#[test]
fn failed_launch_record_rolls_back_session_and_retries_without_double_counting() {
    let mut connection = database();
    connection
        .execute_batch(
            "CREATE TRIGGER fail_count BEFORE UPDATE OF play_count ON games
         BEGIN SELECT RAISE(FAIL, 'forced failure'); END;",
        )
        .unwrap();
    let entry = session("retry", 1_000, 0, None);
    assert!(save(&mut connection, &entry).is_err());
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM play_sessions", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    connection
        .execute_batch("DROP TRIGGER fail_count;")
        .unwrap();
    save(&mut connection, &entry).unwrap();
    save(&mut connection, &entry).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT play_count FROM games", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

// 测试场景：旧库升级保留原启动次数，重复迁移安全；移除游戏同时清理其会话。
#[test]
fn migration_preserves_existing_game_data_and_deletion_cleans_sessions() {
    let mut connection = database();
    connection
        .execute_batch(
            "DROP TABLE play_sessions; UPDATE games SET play_count = 7, last_play_time = 123;",
        )
        .unwrap();
    crate::db::run_migrations(&connection).unwrap();
    crate::db::run_migrations(&connection).unwrap();
    let game = super::super::repository::get_by_id(&connection, "game")
        .unwrap()
        .unwrap();
    assert_eq!(game.play_count, 7);
    assert_eq!(game.last_play_time, Some(123));
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM play_sessions", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    save(
        &mut connection,
        &session("saved", 1_000, 30_000, Some(EndReason::GameExited)),
    )
    .unwrap();
    super::super::repository::delete(&connection, "game").unwrap();
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM play_sessions", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

// 测试场景：旧启动记录没有可推算的时长；零时长新会话与“无记录”明确区分。
#[test]
fn summaries_distinguish_missing_history_from_known_zero_duration() {
    let mut connection = database();
    connection
        .execute("UPDATE games SET play_count = 7", [])
        .unwrap();
    let summary = summaries(&connection).unwrap().remove(0);
    assert_eq!(summary.total_duration_ms, None);
    assert_eq!(summary.last_session_duration_ms, None);
    save(
        &mut connection,
        &session("zero", 1_000, 0, Some(EndReason::GameExited)),
    )
    .unwrap();
    let summary = summaries(&connection).unwrap().remove(0);
    assert_eq!(summary.total_duration_ms, Some(0));
    assert_eq!(summary.last_session_duration_ms, Some(0));
}

// 测试场景：跨会话汇总，只替换本次已落盘部分；回拨系统时间不影响最近会话选择。
#[test]
fn summaries_overlay_live_progress_without_counting_checkpoints_twice() {
    let mut connection = database();
    save(
        &mut connection,
        &session("previous", 100_000, 60_000, Some(EndReason::ManagerExited)),
    )
    .unwrap();
    let mut current = session("current", 1_000, 30_000, None);
    save(&mut connection, &current).unwrap();
    current.duration_ms = 45_000;
    let mut summary = summaries(&connection).unwrap().remove(0);
    summary.with_live_session(&current);
    assert_eq!(summary.total_duration_ms, Some(105_000));
    assert_eq!(summary.last_session_duration_ms, Some(45_000));
    assert_eq!(summary.current_session_duration_ms, Some(45_000));
    current.end_reason = Some(EndReason::Interrupted);
    save(&mut connection, &current).unwrap();
    let summary = summaries(&connection).unwrap().remove(0);
    assert_eq!(summary.total_duration_ms, Some(105_000));
    assert_eq!(summary.last_session_duration_ms, Some(45_000));
    assert_eq!(
        summary.last_session_end_reason,
        Some(EndReason::Interrupted)
    );
    assert_eq!(summary.current_session_duration_ms, None);
}

// 测试场景：新会话首次落盘失败，展示仍包含已确认的本次进度，旧时长不被减掉。
#[test]
fn summaries_include_live_sessions_not_yet_persisted() {
    let mut connection = database();
    save(
        &mut connection,
        &session("previous", 1_000, 60_000, Some(EndReason::GameExited)),
    )
    .unwrap();
    let mut summary = summaries(&connection).unwrap().remove(0);
    summary.with_live_session(&session("unsaved", 100_000, 12_000, None));
    summary.with_live_session(&session("unsaved", 100_000, 12_000, None));
    assert_eq!(summary.total_duration_ms, Some(72_000));
    assert_eq!(summary.last_session_duration_ms, Some(12_000));
    assert_eq!(summary.current_session_duration_ms, Some(12_000));
}
