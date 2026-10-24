use super::*;
use std::sync::{atomic::AtomicU8, Arc};

struct FakeProcess(Arc<AtomicU8>);
impl TrackedProcess for FakeProcess {
    fn has_exited(&mut self) -> std::io::Result<bool> {
        match self.0.load(Ordering::SeqCst) {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(std::io::Error::other("query failed")),
        }
    }
}

struct Fixture {
    registry: RunningGames,
    path: PathBuf,
    status: Arc<AtomicU8>,
    started: Instant,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "game-shift-timing-{}-{}.sqlite3",
            std::process::id(),
            NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
        ));
        let connection = crate::db::open_path(&path).unwrap();
        crate::db::run_migrations(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO games (id, name, exe_path, folder_path, create_time, update_time)
             VALUES ('game', 'Example', 'game.exe', '.', 1, 1)",
                [],
            )
            .unwrap();
        let registry = RunningGames::default();
        registry.initialize_database(path.clone()).unwrap();
        let status = Arc::new(AtomicU8::new(0));
        registry
            .start("game", || Ok(Box::new(FakeProcess(status.clone()))))
            .unwrap();
        let started = registry.0.lock().unwrap().games["game"].started;
        Self {
            registry,
            path,
            status,
            started,
        }
    }

    fn refresh(&self, milliseconds: u64, closing: bool) {
        self.registry
            .0
            .lock()
            .unwrap()
            .refresh(self.started + Duration::from_millis(milliseconds), closing);
    }

    fn saved(&self) -> (i64, Option<String>) {
        crate::db::open_path(&self.path).unwrap().query_row(
            "SELECT duration_ms, end_reason FROM play_sessions ORDER BY started_at DESC LIMIT 1", [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.registry.0.lock().unwrap().ownership.take();
        std::fs::remove_file(&self.path).unwrap();
        std::fs::remove_file(self.path.with_extension("tracking.lock")).unwrap();
    }
}

// 测试场景：无需前端轮询也能保存进度，退出按最后确认的存活时刻结算。
#[test]
fn backend_checkpoints_and_settles_game_exit_without_frontend() {
    let fixture = Fixture::new();
    fixture.refresh(29_000, false);
    assert_eq!(fixture.saved(), (0, None));
    fixture.refresh(30_000, false);
    assert_eq!(fixture.saved(), (30_000, None));
    fixture.refresh(45_000, false);
    let running = fixture.registry.snapshot().unwrap();
    assert_eq!(running.playtimes[0].total_duration_ms, Some(45_000));
    assert_eq!(
        running.playtimes[0].current_session_duration_ms,
        Some(45_000)
    );
    fixture.status.store(1, Ordering::SeqCst);
    fixture.refresh(46_000, false);
    assert_eq!(fixture.saved(), (45_000, Some("game_exited".into())));
    assert!(fixture.registry.0.lock().unwrap().games.is_empty());
    let exited = fixture.registry.snapshot().unwrap();
    assert_eq!(exited.playtimes[0].total_duration_ms, Some(45_000));
    assert_eq!(exited.playtimes[0].last_session_duration_ms, Some(45_000));
    assert_eq!(exited.playtimes[0].current_session_duration_ms, None);
}

// 测试场景：管理器先退出时立即保存尾段，保留游戏进程且拒绝继续启动。
#[test]
fn manager_exit_saves_tail_and_stops_tracking_without_killing_process() {
    let fixture = Fixture::new();
    fixture.refresh(12_345, true);
    assert_eq!(fixture.saved(), (12_345, Some("manager_exited".into())));
    assert_eq!(fixture.status.load(Ordering::SeqCst), 0);
    fixture.registry.shutdown().unwrap();
    fixture.registry.shutdown().unwrap();
    assert!(!fixture.registry.tick().unwrap());
    assert!(fixture
        .registry
        .start("game", || panic!("must not spawn while stopping"))
        .is_err());
    assert_eq!(fixture.saved(), (12_345, Some("manager_exited".into())));
}

// 测试场景：保存失败仍保留进程及最终结算点，恢复后重试不额外计算故障期间时长。
#[test]
fn failed_final_save_retains_lock_and_retries_the_frozen_duration() {
    let fixture = Fixture::new();
    let connection = crate::db::open_path(&fixture.path).unwrap();
    connection.execute_batch("CREATE TRIGGER fail_save BEFORE UPDATE ON play_sessions BEGIN SELECT RAISE(FAIL, 'forced failure'); END;").unwrap();
    fixture.refresh(20_000, false);
    fixture.status.store(1, Ordering::SeqCst);
    fixture.refresh(21_000, false);
    assert!(fixture
        .registry
        .0
        .lock()
        .unwrap()
        .games
        .contains_key("game"));
    assert!(fixture
        .registry
        .snapshot()
        .unwrap()
        .error
        .unwrap()
        .contains("保存失败"));
    assert!(fixture
        .registry
        .with_status("game", |_| -> Result<(), String> {
            panic!("must not mutate before saving")
        })
        .is_err());
    connection.execute_batch("DROP TRIGGER fail_save;").unwrap();
    fixture.refresh(80_000, false);
    assert_eq!(fixture.saved(), (20_000, Some("game_exited".into())));
    assert!(fixture.registry.0.lock().unwrap().games.is_empty());
}

// 测试场景：首次写库失败不能丢失已经启动的进程，后台重试只增加一次启动次数。
#[test]
fn initial_save_failure_retains_the_spawned_process_and_recovers() {
    let fixture = Fixture::new();
    fixture.status.store(1, Ordering::SeqCst);
    fixture.refresh(1_000, false);
    let connection = crate::db::open_path(&fixture.path).unwrap();
    connection.execute_batch("CREATE TRIGGER fail_insert BEFORE INSERT ON play_sessions BEGIN SELECT RAISE(FAIL, 'forced failure'); END;").unwrap();
    fixture.status.store(0, Ordering::SeqCst);
    let error = fixture
        .registry
        .start("game", || Ok(Box::new(FakeProcess(fixture.status.clone()))))
        .unwrap_err();
    assert!(error.contains("游戏已启动"));
    assert!(fixture
        .registry
        .start("game", || panic!("no duplicate spawn"))
        .is_err());
    connection
        .execute_batch("DROP TRIGGER fail_insert;")
        .unwrap();
    fixture.registry.tick().unwrap();
    fixture.registry.tick().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT play_count FROM games", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert!(fixture.registry.snapshot().unwrap().error.is_none());
}

// 测试场景：查询异常时不增加已确认时长，管理器退出后标记中断并保留最后确认值。
#[test]
fn unknown_process_status_does_not_extend_time_on_shutdown() {
    let fixture = Fixture::new();
    fixture.refresh(10_000, false);
    fixture.status.store(2, Ordering::SeqCst);
    fixture.refresh(60_000, true);
    assert_eq!(fixture.saved(), (10_000, Some("interrupted".into())));
}

// 测试场景：模拟强制结束后的新会话，不接管旧进程，不补算管理器关闭期间时间。
#[test]
fn restart_recovers_only_persisted_progress() {
    let fixture = Fixture::new();
    fixture.refresh(30_000, false);
    fixture.refresh(45_000, false);
    // 模拟旧进程强制结束后，操作系统释放文件锁。
    fixture.registry.0.lock().unwrap().ownership.take();
    let restarted = RunningGames::default();
    restarted.initialize_database(fixture.path.clone()).unwrap();
    assert_eq!(fixture.saved(), (30_000, Some("interrupted".into())));
    assert!(restarted.snapshot().unwrap().game_ids.is_empty());
}

// 测试场景：另一个管理器实例不能把当前正在计时的记录误结算为中断。
#[test]
fn another_instance_cannot_recover_an_active_managers_records() {
    let fixture = Fixture::new();
    fixture.refresh(30_000, false);
    let another = RunningGames::default();
    assert!(another.initialize_database(fixture.path.clone()).is_err());
    assert_eq!(fixture.saved(), (30_000, None));
}

// 测试场景：Updater 清理资源时也会结算，后续 Exit 回调不会重复结算。
#[test]
fn updater_cleanup_guard_settles_and_stops_tracking() {
    let fixture = Fixture::new();
    fixture.refresh(12_000, false);
    drop(ShutdownGuard(fixture.registry.clone()));
    assert_eq!(fixture.saved(), (12_000, Some("manager_exited".into())));
    fixture.registry.shutdown().unwrap();
    assert_eq!(fixture.saved(), (12_000, Some("manager_exited".into())));
    assert!(!fixture.registry.tick().unwrap());
}
