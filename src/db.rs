//! SQLite connection pool and schema bootstrap. Server-only.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::OnceLock;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Open the database, creating the file and applying migrations if needed.
pub async fn init(database_url: &str) -> Result<(), sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        // WAL keeps the dashboard responsive while a long write is in flight.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    POOL.set(pool)
        .expect("database initialised more than once");
    Ok(())
}

/// The shared pool. Panics if called before [`init`].
pub fn pool() -> &'static SqlitePool {
    POOL.get()
        .expect("database not initialised; call db::init first")
}

/// A fresh record id.
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Current timestamp as an RFC 3339 string, the format every `*_at` column uses.
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Today's date as `YYYY-MM-DD`, used for session log defaults.
pub fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Ensure there is at least one party, so the dashboard has somewhere to put
/// characters on a fresh install.
pub async fn ensure_default_party() -> Result<String, sqlx::Error> {
    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM parties LIMIT 1")
        .fetch_optional(pool())
        .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let id = new_id();
    sqlx::query(
        "INSERT INTO parties (id, owner_id, name, notes, gold, created_at)
         VALUES (?, NULL, ?, '', 0, ?)",
    )
    .bind(&id)
    .bind("The Party")
    .bind(now())
    .execute(pool())
    .await?;

    Ok(id)
}
