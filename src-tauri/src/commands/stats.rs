use crate::repositories::db::DbState;
use serde::Serialize;
use tauri::command;

#[derive(Debug, Serialize)]
pub struct DailyActivity {
    /// Date string in "YYYY-MM-DD" format
    pub date: String,
    /// Number of messages sent by the user that day
    pub message_count: i64,
}

#[derive(Debug, Serialize)]
pub struct LearningStats {
    /// Total user messages ever sent (approximates "conversations")
    pub total_messages: i64,
    /// Total vocabulary items saved
    pub total_vocabulary: i64,
    /// Total grammar corrections run
    pub total_corrections: i64,
    /// Number of unique days with activity in the last 30 days
    pub active_days_30: i64,
    /// Current consecutive-day streak
    pub streak_days: i64,
    /// Per-day activity for the last 7 days (oldest → newest)
    pub daily_activity: Vec<DailyActivity>,
    /// Top corrected words / phrases (original_text, count)
    pub top_corrections: Vec<(String, i64)>,
    /// Most recent saved correction records.
    pub recent_corrections: Vec<RecentCorrection>,
    /// Number of sessions grouped by practice mode.
    pub mode_distribution: Vec<ModeDistribution>,
}

#[derive(Debug, Serialize)]
pub struct RecentCorrection {
    pub id: String,
    pub session_id: Option<String>,
    pub original_text: String,
    pub corrected_text: String,
    pub explanation: Option<String>,
    pub better_expression: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ModeDistribution {
    pub mode_id: String,
    pub mode_name: String,
    pub session_count: i64,
}

#[command]
pub fn get_learning_stats(db: tauri::State<'_, DbState>) -> Result<LearningStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // ── Total user messages ───────────────────────────────────────────
    let total_messages: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE role = 'user'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // ── Total vocabulary ──────────────────────────────────────────────
    let total_vocabulary: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocabulary", [], |row| row.get(0))
        .unwrap_or(0);

    // ── Total corrections ─────────────────────────────────────────────
    let total_corrections: i64 = conn
        .query_row("SELECT COUNT(*) FROM corrections", [], |row| row.get(0))
        .unwrap_or(0);

    // ── Active days in last 30 days ───────────────────────────────────
    // messages.timestamp is stored as Unix epoch (INTEGER)
    let active_days_30: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT date(timestamp / 1000, 'unixepoch')) 
             FROM messages 
             WHERE role = 'user'
               AND timestamp > strftime('%s', 'now', '-30 days') * 1000",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // ── Streak calculation ────────────────────────────────────────────
    // Fetch the last 60 distinct active days (descending) and count
    // the unbroken chain starting from today/yesterday.
    let streak_days = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT date(timestamp / 1000, 'unixepoch') as d
                 FROM messages
                 WHERE role = 'user'
                 ORDER BY d DESC
                 LIMIT 60",
            )
            .map_err(|e| e.to_string())?;

        let dates: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        compute_streak(&dates)
    };

    // ── Daily activity last 7 days ────────────────────────────────────
    let daily_activity = {
        let mut stmt = conn
            .prepare(
                "SELECT date(timestamp / 1000, 'unixepoch') as d, COUNT(*) as cnt
                 FROM messages
                 WHERE role = 'user'
                   AND timestamp > strftime('%s', 'now', '-7 days') * 1000
                 GROUP BY d
                 ORDER BY d ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<DailyActivity> = stmt
            .query_map([], |row| {
                Ok(DailyActivity {
                    date: row.get(0)?,
                    message_count: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        // Fill in missing days so frontend always sees 7 data points
        fill_daily_gaps(rows, 7)
    };

    // ── Top corrections ───────────────────────────────────────────────
    let top_corrections = {
        let mut stmt = conn
            .prepare(
                "SELECT original_text, COUNT(*) as cnt
                 FROM corrections
                 GROUP BY original_text
                 ORDER BY cnt DESC
                 LIMIT 8",
            )
            .map_err(|e| e.to_string())?;

        let x: Vec<(String, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        x
    };

    let recent_corrections = {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, original_text, corrected_text, explanation, better_expression, created_at
                 FROM corrections
                 ORDER BY created_at DESC
                 LIMIT 8",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(RecentCorrection {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    original_text: row.get(2)?,
                    corrected_text: row.get(3)?,
                    explanation: row.get(4)?,
                    better_expression: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.filter_map(|row| row.ok()).collect()
    };

    let mode_distribution = {
        let mut stmt = conn
            .prepare(
                "SELECT
                    COALESCE(s.mode_id, 'free_talk') AS mode_id,
                    COALESCE(pm.name, 'Free Talk') AS mode_name,
                    COUNT(*) AS session_count
                 FROM chat_sessions s
                 LEFT JOIN practice_modes pm ON pm.id = COALESCE(s.mode_id, 'free_talk')
                 GROUP BY mode_id, mode_name
                 ORDER BY session_count DESC, mode_name ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ModeDistribution {
                    mode_id: row.get(0)?,
                    mode_name: row.get(1)?,
                    session_count: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.filter_map(|row| row.ok()).collect()
    };

    Ok(LearningStats {
        total_messages,
        total_vocabulary,
        total_corrections,
        active_days_30,
        streak_days,
        daily_activity,
        top_corrections,
        recent_corrections,
        mode_distribution,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Given a descending list of "YYYY-MM-DD" strings, return the streak length
/// (consecutive days including today or yesterday, otherwise 0).
fn compute_streak(dates: &[String]) -> i64 {
    if dates.is_empty() {
        return 0;
    }
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // Streak must start with today or yesterday
    if dates[0] != today && dates[0] != yesterday {
        return 0;
    }

    let mut streak = 1i64;
    for window in dates.windows(2) {
        let a = chrono::NaiveDate::parse_from_str(&window[0], "%Y-%m-%d");
        let b = chrono::NaiveDate::parse_from_str(&window[1], "%Y-%m-%d");
        if let (Ok(a), Ok(b)) = (a, b) {
            if (a - b).num_days() == 1 {
                streak += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    streak
}

/// Ensure `days` consecutive dates ending today are represented.
/// Missing days get a message_count of 0.
fn fill_daily_gaps(existing: Vec<DailyActivity>, days: u32) -> Vec<DailyActivity> {
    let today = chrono::Utc::now();
    let mut result = Vec::with_capacity(days as usize);

    for i in (0..days).rev() {
        let date = (today - chrono::Duration::days(i as i64))
            .format("%Y-%m-%d")
            .to_string();
        let count = existing
            .iter()
            .find(|d| d.date == date)
            .map(|d| d.message_count)
            .unwrap_or(0);
        result.push(DailyActivity {
            date,
            message_count: count,
        });
    }
    result
}
