use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub name: String,
    pub gender: String,
    pub age_group: String,
    pub occupation: String,
    pub industry: String,
    pub language: String,
}

pub fn init_profile_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_profile (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_interests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT DEFAULT 'default' NOT NULL,
            interest TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, interest)
        )",
        [],
    )?;

    Ok(())
}

pub fn get_profile_value(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM user_profile WHERE key = ?")?;
    let mut rows = stmt.query([key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_profile_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO user_profile (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
        [key, value],
    )?;
    Ok(())
}

pub fn get_interests(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT interest FROM user_interests WHERE user_id = 'default'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut interests = Vec::new();
    for interest in rows {
        interests.push(interest?);
    }
    Ok(interests)
}

pub fn get_user_profile(conn: &Connection) -> Result<UserProfile, rusqlite::Error> {
    Ok(UserProfile {
        name: get_profile_value(conn, "name")?.unwrap_or_default(),
        gender: get_profile_value(conn, "gender")?.unwrap_or_default(),
        age_group: get_profile_value(conn, "age_group")?.unwrap_or_default(),
        occupation: get_profile_value(conn, "occupation")?.unwrap_or_default(),
        industry: get_profile_value(conn, "industry")?.unwrap_or_default(),
        language: get_profile_value(conn, "language")?.unwrap_or_else(|| "auto".to_string()),
    })
}

pub fn add_interest(conn: &Connection, interest: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_interests (user_id, interest) VALUES ('default', ?)",
        [interest],
    )?;
    Ok(())
}

pub fn remove_interest(conn: &Connection, interest: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM user_interests WHERE user_id = 'default' AND interest = ?",
        [interest],
    )?;
    Ok(())
}
