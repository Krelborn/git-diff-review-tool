use std::sync::Mutex;

use serde::Serialize;
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};

const INITIAL_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS repos (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  path     TEXT NOT NULL UNIQUE,
  name     TEXT NOT NULL,
  added_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS comments (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id    INTEGER NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
  file_path  TEXT NOT NULL,
  line_num   INTEGER NOT NULL,
  body       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_comments_repo ON comments(repo_id);
CREATE INDEX IF NOT EXISTS idx_comments_file ON comments(repo_id, file_path);
";

/// A repository entry returned from the database.
#[derive(Clone, Debug, Serialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub path: String,
}

/// Managed state holding the single rusqlite connection.
///
/// rusqlite::Connection is Send but not Sync; wrapping it in a Mutex makes it
/// safe to share across Tauri command threads.
pub struct DbState(Mutex<rusqlite::Connection>);

#[tauri::command]
fn db_health_check() -> &'static str {
    "ok"
}

/// Returns all repositories ordered by the time they were added.
#[tauri::command]
fn list_repos(state: tauri::State<'_, DbState>) -> Result<Vec<Repo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, path, name FROM repos ORDER BY added_at")
        .map_err(|e| e.to_string())?;

    let repos = stmt
        .query_map([], |row| {
            Ok(Repo {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(repos)
}

/// Validates that `path` is a git repository, then inserts it into the database.
///
/// Returns the newly created `Repo` on success, or a human-readable error string
/// if the path is not a git repository or has already been added.
#[tauri::command]
fn add_repo(path: String, state: tauri::State<'_, DbState>) -> Result<Repo, String> {
    // Confirm the path points to a git working tree before touching the DB.
    let status = std::process::Command::new("git")
        .args(["-C", &path, "rev-parse", "--is-inside-work-tree"])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("Not a git repository".to_string());
    }

    // Use the last path segment as a human-readable name, falling back to the
    // full path if the segment cannot be extracted.
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&path)
        .to_string();

    let added_at = chrono::Utc::now().to_rfc3339();

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO repos (path, name, added_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![path, name, added_at],
    )
    .map_err(|e| {
        // SQLite UNIQUE constraint violations surface as a SqliteFailure with
        // extended error code 2067 (SQLITE_CONSTRAINT_UNIQUE). Map that to a
        // friendlier message rather than leaking internal DB error text.
        if let rusqlite::Error::SqliteFailure(ref sqlite_err, _) = e {
            if sqlite_err.extended_code == 2067 {
                return "Repository already added".to_string();
            }
        }
        e.to_string()
    })?;

    let id = conn.last_insert_rowid();

    Ok(Repo { id, name, path })
}

/// Removes a repository by its primary key. The ON DELETE CASCADE constraint
/// in the schema ensures associated comments are removed automatically.
#[tauri::command]
fn remove_repo(id: i64, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM repos WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_schema",
        sql: INITIAL_SCHEMA,
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            SqlBuilder::new()
                .add_migrations("sqlite:review-tool.db", migrations)
                .build(),
        )
        .setup(|app| {
            use tauri::Manager;
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("review-tool.db");
            let conn = rusqlite::Connection::open(&db_path)?;
            conn.execute_batch(INITIAL_SCHEMA)?;
            app.manage(DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db_health_check,
            list_repos,
            add_repo,
            remove_repo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
