use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use super::sessions::{self, EndReason, PlaySession};

const CHECKPOINT_MS: i64 = 30_000;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);

trait TrackedProcess: Send {
    fn has_exited(&mut self) -> std::io::Result<bool>;
}

impl TrackedProcess for Child {
    fn has_exited(&mut self) -> std::io::Result<bool> {
        self.try_wait().map(|status| status.is_some())
    }
}

#[derive(Clone, Default)]
pub(crate) struct RunningGames(Arc<Mutex<RuntimeState>>);

// Windows 安装更新时 Updater 会直接退出进程，不经过 RunEvent::Exit，
// 但会先清理 Tauri 资源；使用同一结算逻辑保留这一退出路径的尾段。
struct ShutdownGuard(RunningGames);

impl tauri::Resource for ShutdownGuard {}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        if let Err(error) = self.0.shutdown() {
            eprintln!("退出时保存游玩记录失败：{error}");
        }
    }
}

#[derive(Default)]
struct RuntimeState {
    games: HashMap<String, RunningGame>,
    database: Option<PathBuf>,
    // 保持独占文件锁，防止另一个实例把当前活跃会话当成崩溃遗留记录恢复。
    ownership: Option<std::fs::File>,
    stopping: bool,
    error: Option<String>,
}

struct RunningGame {
    process: Box<dyn TrackedProcess>,
    session: PlaySession,
    started: Instant,
    last_saved_ms: Option<i64>,
}

impl RunningGame {
    fn observe(&mut self, now: Instant, closing: bool) -> Result<(), String> {
        if self.session.end_reason.is_some() {
            return Ok(());
        }
        match self.process.has_exited() {
            Ok(false) => {
                self.session.duration_ms = self.session.duration_ms.max(
                    i64::try_from(now.saturating_duration_since(self.started).as_millis())
                        .unwrap_or(i64::MAX),
                );
                if closing {
                    self.session.end_reason = Some(EndReason::ManagerExited);
                }
            }
            // 进程退出发生在两次检查之间，保守结算到最后确认存活的时刻。
            Ok(true) => self.session.end_reason = Some(EndReason::GameExited),
            Err(_) => {
                if closing {
                    self.session.end_reason = Some(EndReason::Interrupted);
                    return Ok(());
                }
                return Err("部分游戏状态暂时无法确认，已保留启动锁定，稍后自动重试".into());
            }
        }
        Ok(())
    }

    fn save_if_due(
        &mut self,
        database: Option<&std::path::Path>,
        force: bool,
    ) -> Result<(), String> {
        let due = self
            .last_saved_ms
            .is_none_or(|last| self.session.duration_ms - last >= CHECKPOINT_MS);
        if !force && !due && self.session.end_reason.is_none() {
            return Ok(());
        }
        if let Some(path) = database {
            let result = crate::db::open_path(path)
                .and_then(|mut connection| sessions::save(&mut connection, &self.session));
            result.map_err(|error| format!("游玩记录保存失败，将自动重试：{error}"))?;
        }
        self.last_saved_ms = Some(self.session.duration_ms);
        Ok(())
    }
}

impl RuntimeState {
    fn refresh_game(&mut self, id: &str, now: Instant, closing: bool) -> Result<(), String> {
        let Some(game) = self.games.get_mut(id) else {
            return Ok(());
        };
        let observed = game.observe(now, closing);
        game.save_if_due(self.database.as_deref(), closing)?;
        if game.session.end_reason.is_some() {
            self.games.remove(id);
        }
        observed
    }

    fn refresh(&mut self, now: Instant, closing: bool) {
        self.error = None;
        for id in self.games.keys().cloned().collect::<Vec<_>>() {
            if let Err(error) = self.refresh_game(&id, now, closing) {
                self.error = Some(error);
            }
        }
    }
}

pub(crate) fn initialize(app: &AppHandle) -> Result<(), String> {
    let registry = app.state::<RunningGames>();
    registry.initialize_database(crate::db::database_path(app)?)?;
    app.resources_table()
        .add(ShutdownGuard(registry.inner().clone()));
    let handle = app.clone();
    std::thread::Builder::new()
        .name("game-playtime".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let registry = handle.state::<RunningGames>();
            match registry.tick() {
                Ok(true) => (),
                Ok(false) => break,
                Err(error) => eprintln!("游玩计时任务失败：{error}"),
            }
        })
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunningSnapshot {
    game_ids: Vec<String>,
    error: Option<String>,
}

impl RunningGames {
    fn initialize_database(&self, path: PathBuf) -> Result<(), String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        if state.database.is_some() {
            return Err("游玩计时已经初始化".into());
        }
        let ownership = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("tracking.lock"))
            .map_err(|error| error.to_string())?;
        ownership.try_lock().map_err(|error| {
            format!("无法独占游玩记录，请确认其他 Game Shift 实例已退出：{error}")
        })?;
        let connection = crate::db::open_path(&path)?;
        sessions::recover_interrupted(&connection)?;
        state.database = Some(path);
        state.ownership = Some(ownership);
        Ok(())
    }

    fn tick(&self) -> Result<bool, String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        if state.stopping {
            return Ok(false);
        }
        state.refresh(Instant::now(), false);
        Ok(true)
    }

    pub(crate) fn shutdown(&self) -> Result<(), String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        state.stopping = true;
        state.refresh(Instant::now(), true);
        state.error.clone().map_or(Ok(()), Err)
    }

    pub(super) fn spawn(
        &self,
        id: &str,
        prepare: impl FnOnce() -> Result<Command, String>,
    ) -> Result<(), String> {
        self.start(id, || {
            prepare()?
                .spawn()
                .map(|child| Box::new(child) as Box<dyn TrackedProcess>)
                .map_err(|error| super::launcher::format_launch_error(&error))
        })
    }

    // 启动与最终写库共享锁；不能只在耗时的封面处理之前检查一次。
    pub(super) fn with_status<T>(
        &self,
        id: &str,
        action: impl FnOnce(bool) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        state.refresh_game(id, Instant::now(), false)?;
        let running = state.games.contains_key(id);
        action(running)
    }

    fn start(
        &self,
        id: &str,
        spawn: impl FnOnce() -> Result<Box<dyn TrackedProcess>, String>,
    ) -> Result<(), String> {
        // 检查和创建进程使用同一把锁，避免同时收到两个启动请求时重复 spawn。
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        if state.stopping {
            return Err("Game Shift 正在退出，无法启动游戏".into());
        }
        state.refresh_game(id, Instant::now(), false)?;
        if state.games.contains_key(id) {
            return Err("游戏正在运行，请勿重复启动".to_string());
        }
        let started_at = super::current_timestamp_millis()?;
        let preparing = Instant::now();
        let session_id = format!(
            "{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_nanos(),
            NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
        );
        // 先完成所有可能失败的元数据准备；进程启动后必须保留句柄。
        let process = spawn()?;
        let started = Instant::now();
        let started_at = started_at.saturating_add(
            i64::try_from(started.duration_since(preparing).as_millis()).unwrap_or(i64::MAX),
        );
        state.games.insert(
            id.to_string(),
            RunningGame {
                process,
                session: PlaySession {
                    id: session_id,
                    game_id: id.into(),
                    started_at,
                    duration_ms: 0,
                    end_reason: None,
                },
                started,
                last_saved_ms: None,
            },
        );
        let database = state.database.clone();
        if let Err(error) = state
            .games
            .get_mut(id)
            .expect("newly inserted game")
            .save_if_due(database.as_deref(), true)
        {
            state.error = Some(error.clone());
            return Err(format!("游戏已启动，但{error}"));
        }
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> Result<RunningSnapshot, String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        if !state.stopping {
            state.refresh(Instant::now(), false);
        }
        let mut game_ids = state.games.keys().cloned().collect::<Vec<_>>();
        game_ids.sort();
        Ok(RunningSnapshot {
            game_ids,
            error: state.error.clone(),
        })
    }
}

#[cfg(test)]
#[path = "runtime_timing_tests.rs"]
mod timing_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    };

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

    #[test]
    fn blocks_duplicates_until_exit_and_allows_relaunch() {
        let registry = RunningGames::default();
        let status = Arc::new(AtomicU8::new(0));
        registry
            .start("game", || Ok(Box::new(FakeProcess(status.clone()))))
            .unwrap();
        assert_eq!(registry.snapshot().unwrap().game_ids, vec!["game"]);
        assert!(registry
            .start("game", || panic!("duplicate spawn"))
            .is_err());
        status.store(1, Ordering::SeqCst);
        assert!(registry.snapshot().unwrap().game_ids.is_empty());
        registry
            .start("game", || {
                Ok(Box::new(FakeProcess(Arc::new(AtomicU8::new(0)))))
            })
            .unwrap();
        assert_eq!(registry.snapshot().unwrap().game_ids, vec!["game"]);
        // 新应用会话没有旧的持久化运行标记。
        assert!(RunningGames::default()
            .snapshot()
            .unwrap()
            .game_ids
            .is_empty());
    }

    #[test]
    fn query_failure_preserves_lock_and_recovers() {
        let registry = RunningGames::default();
        let status = Arc::new(AtomicU8::new(2));
        registry
            .start("game", || Ok(Box::new(FakeProcess(status.clone()))))
            .unwrap();
        let snapshot = registry.snapshot().unwrap();
        assert!(snapshot.error.is_some());
        assert_eq!(snapshot.game_ids, vec!["game"]);
        assert!(registry
            .start("game", || panic!("unknown process must not restart"))
            .is_err());
        status.store(1, Ordering::SeqCst);
        let snapshot = registry.snapshot().unwrap();
        assert!(snapshot.error.is_none());
        assert!(snapshot.game_ids.is_empty());
    }

    #[test]
    fn failed_spawn_does_not_leave_running_state() {
        let registry = RunningGames::default();
        assert!(registry
            .start("game", || Err("spawn failed".into()))
            .is_err());
        assert!(registry.snapshot().unwrap().game_ids.is_empty());
    }

    #[test]
    fn mutation_checks_current_status_and_recovers_after_exit() {
        let registry = RunningGames::default();
        let status = Arc::new(AtomicU8::new(0));
        registry
            .start("game", || Ok(Box::new(FakeProcess(status.clone()))))
            .unwrap();
        assert!(registry
            .with_status("game", |running| if running {
                Err::<(), _>("blocked".into())
            } else {
                Ok(())
            })
            .is_err());
        assert_eq!(registry.snapshot().unwrap().game_ids, vec!["game"]);
        status.store(1, Ordering::SeqCst);
        assert!(registry
            .with_status("game", |running| {
                assert!(!running);
                Ok(())
            })
            .is_ok());
    }

    #[test]
    fn concurrent_launch_requests_spawn_only_once() {
        let registry = Arc::new(RunningGames::default());
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let calls = Arc::new(AtomicU8::new(0));
        let threads = (0..2)
            .map(|_| {
                let registry = registry.clone();
                let barrier = barrier.clone();
                let calls = calls.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    registry.start("game", || {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Box::new(FakeProcess(Arc::new(AtomicU8::new(0)))))
                    })
                })
            })
            .collect::<Vec<_>>();
        let successes = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .filter(Result::is_ok)
            .count();
        assert_eq!(successes, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn real_child_exit_is_reaped() {
        // 启动测试程序的 --list 模式，不启动用户游戏，也不打开控制台窗口。
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .arg("--list")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut child = command.spawn().unwrap();
        child.wait().unwrap();
        let registry = RunningGames::default();
        registry.start("game", || Ok(Box::new(child))).unwrap();
        assert!(registry.snapshot().unwrap().game_ids.is_empty());
    }
}
