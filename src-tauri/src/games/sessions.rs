use rusqlite::{params, Connection};

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GamePlaytime {
    pub game_id: String,
    pub total_duration_ms: Option<i64>,
    pub last_session_duration_ms: Option<i64>,
    pub last_session_end_reason: Option<EndReason>,
    pub current_session_duration_ms: Option<i64>,
    #[serde(skip)]
    last_session_id: Option<String>,
}

impl GamePlaytime {
    pub fn with_live_session(&mut self, session: &PlaySession) {
        let persisted_current = if self.last_session_id.as_deref() == Some(&session.id) {
            self.last_session_duration_ms.unwrap_or(0)
        } else {
            0
        };
        self.total_duration_ms = Some(
            self.total_duration_ms
                .unwrap_or(0)
                .saturating_sub(persisted_current)
                .saturating_add(session.duration_ms),
        );
        self.last_session_duration_ms = Some(session.duration_ms);
        self.last_session_id = Some(session.id.clone());
        self.last_session_end_reason = session.end_reason;
        self.current_session_duration_ms =
            session.end_reason.is_none().then_some(session.duration_ms);
    }
}

pub(super) fn summaries(connection: &Connection) -> Result<Vec<GamePlaytime>, String> {
    // rowid 代表会话创建顺序，系统时间回拨也不会让旧会话变成“最近一次”。
    let mut statement = connection
        .prepare(
            "WITH totals AS (
            SELECT game_id, SUM(duration_ms) AS total_ms, MAX(rowid) AS last_row
            FROM play_sessions GROUP BY game_id
         )
         SELECT g.id, t.total_ms, s.duration_ms, s.end_reason, s.id
         FROM games g
         LEFT JOIN totals t ON t.game_id = g.id
         LEFT JOIN play_sessions s ON s.rowid = t.last_row
         ORDER BY g.id",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            let reason: Option<String> = row.get(3)?;
            let last_session_end_reason = match reason.as_deref() {
                Some("game_exited") => Some(EndReason::GameExited),
                Some("manager_exited") => Some(EndReason::ManagerExited),
                Some("interrupted") => Some(EndReason::Interrupted),
                _ => None,
            };
            Ok(GamePlaytime {
                game_id: row.get(0)?,
                total_duration_ms: row.get(1)?,
                last_session_duration_ms: row.get(2)?,
                last_session_end_reason,
                current_session_duration_ms: None,
                last_session_id: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
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
