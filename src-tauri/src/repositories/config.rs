use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::{params, OptionalExtension};

pub type LegacyApiConfig = (Option<String>, Option<String>, Option<String>);

#[derive(Debug, Clone)]
pub struct ConfigProfileRecord {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub is_default: bool,
}

/// Get a config value by key. Returns None if key doesn't exist.
pub fn get_config_value(db: &DbState, key: &str) -> Result<Option<String>, AppError> {
    let conn = db.lock_conn()?;
    let mut stmt = conn.prepare("SELECT value FROM app_config WHERE key = ?1")?;
    let result = stmt.query_row(params![key], |row| row.get(0)).ok();
    Ok(result)
}

fn save_config_value_with_conn(
    conn: &rusqlite::Connection,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn row_to_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConfigProfileRecord> {
    let is_default: i64 = row.get(5)?;
    Ok(ConfigProfileRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        provider: row.get(2)?,
        base_url: row.get(3)?,
        model: row.get(4)?,
        is_default: is_default == 1,
    })
}

/// Save all API profile fields except API key, which lives in the OS keychain.
pub fn save_api_config(
    db: &DbState,
    profile_id: &str,
    profile_name: &str,
    provider: &str,
    base_url: &str,
    model: &str,
    make_default: bool,
) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    let now = chrono::Utc::now().to_rfc3339();

    if make_default {
        conn.execute("UPDATE provider_profiles SET is_default = 0", [])?;
    }

    conn.execute(
        "INSERT INTO provider_profiles
            (id, name, provider, base_url, model, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            provider = excluded.provider,
            base_url = excluded.base_url,
            model = excluded.model,
            is_default = CASE WHEN ?6 = 1 THEN 1 ELSE provider_profiles.is_default END,
            updated_at = excluded.updated_at",
        params![
            profile_id,
            profile_name,
            provider,
            base_url,
            model,
            if make_default { 1 } else { 0 },
            now,
        ],
    )?;

    if make_default {
        save_config_value_with_conn(&conn, "active_profile_id", profile_id)?;
        save_config_value_with_conn(&conn, "provider", provider)?;
        save_config_value_with_conn(&conn, "base_url", base_url)?;
        save_config_value_with_conn(&conn, "model", model)?;
    }

    Ok(())
}

pub fn list_config_profiles(db: &DbState) -> Result<Vec<ConfigProfileRecord>, AppError> {
    let conn = db.lock_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         ORDER BY is_default DESC, updated_at DESC, name ASC",
    )?;
    let rows = stmt.query_map([], row_to_profile)?;
    let mut profiles = Vec::new();

    for row in rows {
        profiles.push(row?);
    }

    Ok(profiles)
}

pub fn get_config_profile(
    db: &DbState,
    profile_id: &str,
) -> Result<Option<ConfigProfileRecord>, AppError> {
    let conn = db.lock_conn()?;
    conn.query_row(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         WHERE id = ?1",
        params![profile_id],
        row_to_profile,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn get_config_profile_by_name(
    db: &DbState,
    profile_name: &str,
) -> Result<Option<ConfigProfileRecord>, AppError> {
    let conn = db.lock_conn()?;
    conn.query_row(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         WHERE lower(name) = lower(?1)
         LIMIT 1",
        params![profile_name.trim()],
        row_to_profile,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn get_fallback_config_profile(
    db: &DbState,
    excluded_profile_id: &str,
    preferred_profile_id: &str,
) -> Result<Option<ConfigProfileRecord>, AppError> {
    let conn = db.lock_conn()?;
    conn.query_row(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         WHERE id <> ?1
         ORDER BY
            CASE WHEN id = ?2 THEN 0 ELSE 1 END,
            updated_at DESC,
            name ASC
         LIMIT 1",
        params![excluded_profile_id, preferred_profile_id],
        row_to_profile,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn rename_config_profile(
    db: &DbState,
    profile_id: &str,
    profile_name: &str,
) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    conn.execute(
        "UPDATE provider_profiles SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![
            profile_id,
            profile_name.trim(),
            chrono::Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn delete_config_profile(db: &DbState, profile_id: &str) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    conn.execute(
        "DELETE FROM provider_profiles WHERE id = ?1",
        params![profile_id],
    )?;
    Ok(())
}

pub fn load_active_config_profile(db: &DbState) -> Result<Option<ConfigProfileRecord>, AppError> {
    let conn = db.lock_conn()?;
    conn.query_row(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         WHERE is_default = 1
         ORDER BY updated_at DESC
         LIMIT 1",
        [],
        row_to_profile,
    )
    .optional()
    .map_err(AppError::from)
}

/// Legacy loader kept for fallback compatibility with databases created before provider profiles.
pub fn load_api_config(db: &DbState) -> Result<LegacyApiConfig, AppError> {
    Ok((
        get_config_value(db, "provider")?,
        get_config_value(db, "base_url")?,
        get_config_value(db, "model")?,
    ))
}

pub fn set_active_config_profile(
    db: &DbState,
    profile: &ConfigProfileRecord,
) -> Result<(), AppError> {
    let conn = db.lock_conn()?;

    conn.execute("UPDATE provider_profiles SET is_default = 0", [])?;
    conn.execute(
        "UPDATE provider_profiles SET is_default = 1, updated_at = ?2 WHERE id = ?1",
        params![profile.id, chrono::Utc::now().to_rfc3339()],
    )?;

    save_config_value_with_conn(&conn, "active_profile_id", &profile.id)?;
    save_config_value_with_conn(&conn, "provider", &profile.provider)?;
    save_config_value_with_conn(&conn, "base_url", &profile.base_url)?;
    save_config_value_with_conn(&conn, "model", &profile.model)?;

    Ok(())
}
