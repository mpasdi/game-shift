use rusqlite::{params, Connection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum EndReason {
    GameExited,
    ManagerExited,
    Interrupted,
}

impl EndReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::GameExited => "game_exited",
            Self::ManagerExited => "manager_exited",
            Self::Interrupted => "interrupted",
        }
    }
}

pub(super) struct PlaySession {
    pub id: String,
    pub game_id: String,
    pub started_at: i64,
    pub duration_ms: i64,
    pub end_reason: Option<EndReason>,
}

impl PlaySession {
    pub fn checkpoint_at(&self) -> i64 {
        self.started_at.saturating_add(self.duration_ms)
    }
}

// 会话插入、启动次数更新和计时保存使用同一事务。失败后重试同一会话不会重复计数。
pub(super) fn save(connection: &mut Connection, session: &PlaySession) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let inserted = transaction
        .execute(
            "INSERT INTO play_sessions (id, game_id, started_at, checkpoint_at)
             VALUES (?1, ?2, ?3, ?3) ON CONFLICT(id) DO NOTHING",
            params![session.id, session.game_id, session.started_at],
        )
        .map_err(|error| error.to_string())?;
    if inserted > 0 {
        super::repository::record_launch(&transaction, &session.game_id, session.started_at)?;
    }
    transaction
        .execute(
            "UPDATE play_sessions SET duration_ms = MAX(duration_ms, ?1),
                checkpoint_at = MAX(checkpoint_at, ?2), ended_at = ?3, end_reason = ?4
             WHERE id = ?5 AND ended_at IS NULL",
            params![
                session.duration_ms,
                session.checkpoint_at(),
                session.end_reason.map(|_| session.checkpoint_at()),
                session.end_reason.map(EndReason::as_str),
                session.id
            ],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

// 重启仅结算到最后一次落盘的检查点，不使用本次启动时间补算离线区间。
pub(super) fn recover_interrupted(connection: &Connection) -> Result<(), String> {
    connection
        .execute(
            "UPDATE play_sessions SET ended_at = checkpoint_at, end_reason = 'interrupted'
             WHERE ended_at IS NULL",
            [],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
#[path = "sessions_tests.rs"]
mod tests;
