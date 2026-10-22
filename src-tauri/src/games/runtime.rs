use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

trait TrackedProcess: Send {
    fn has_exited(&mut self) -> std::io::Result<bool>;
}

impl TrackedProcess for Child {
    fn has_exited(&mut self) -> std::io::Result<bool> {
        self.try_wait().map(|status| status.is_some())
    }
}

#[derive(Default)]
pub(crate) struct RunningGames(Mutex<HashMap<String, Box<dyn TrackedProcess>>>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunningSnapshot {
    game_ids: Vec<String>,
    error: Option<String>,
}

impl RunningGames {
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
        let mut processes = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        let running = match processes.get_mut(id) {
            Some(process) => match process.has_exited() {
                Ok(true) => {
                    processes.remove(id);
                    false
                }
                Ok(false) => true,
                Err(_) => return Err("无法确认游戏是否退出，请稍后重试".to_string()),
            },
            None => false,
        };
        action(running)
    }

    fn start(
        &self,
        id: &str,
        spawn: impl FnOnce() -> Result<Box<dyn TrackedProcess>, String>,
    ) -> Result<(), String> {
        // 检查和创建进程使用同一把锁，避免同时收到两个启动请求时重复 spawn。
        let mut processes = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        if let Some(process) = processes.get_mut(id) {
            match process.has_exited() {
                Ok(true) => {
                    processes.remove(id);
                }
                Ok(false) => return Err("游戏正在运行，请勿重复启动".to_string()),
                Err(_) => return Err("无法确认游戏是否退出，请稍后重试".to_string()),
            }
        }
        processes.insert(id.to_string(), spawn()?);
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> Result<RunningSnapshot, String> {
        let mut processes = self
            .0
            .lock()
            .map_err(|_| "无法读取游戏运行状态".to_string())?;
        let mut error = None;
        processes.retain(|_, child| match child.has_exited() {
            Ok(exited) => !exited,
            Err(_) => {
                error = Some("部分游戏状态暂时无法确认，已保留启动锁定，稍后自动重试".to_string());
                true
            }
        });
        let mut game_ids = processes.keys().cloned().collect::<Vec<_>>();
        game_ids.sort();
        Ok(RunningSnapshot { game_ids, error })
    }
}

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
