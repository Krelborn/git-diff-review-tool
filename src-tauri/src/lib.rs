use std::sync::Mutex;

use serde::{Deserialize, Serialize};
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

/// A comment anchored to a specific line in a diff.
///
/// `is_outdated` is always `false` here — the frontend computes the real value
/// after comparing the line number against the current diff.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: i64,
    pub repo_id: i64,
    pub file_path: String,
    pub line_num: i64,
    pub body: String,
    pub is_outdated: bool,
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

/// Describes which set of changes to diff against.
///
/// Mirrors the TypeScript discriminated union:
/// `type DiffMode = { type: "working-tree" } | { type: "branch"; baseBranch: string }`
///
/// Serde's `tag = "type"` maps the `type` key to the enum variant, and
/// `rename_all = "camelCase"` applies to *field* names within each variant.
/// The variant discriminants themselves are renamed explicitly because
/// `"working-tree"` cannot be expressed by any automatic rename strategy.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DiffMode {
    /// Diff the working tree (staged + unstaged changes) against HEAD.
    #[serde(rename = "working-tree")]
    WorkingTree,

    /// Diff the current HEAD against the point where it diverged from `base_branch`.
    #[serde(rename = "branch")]
    Branch {
        /// The branch name to use as the comparison base (maps to `"baseBranch"` in JSON).
        base_branch: String,
    },
}

/// Returns the unified diff for the repository at `repo_path`.
///
/// - `WorkingTree` mode: `git diff HEAD` — all staged and unstaged changes relative to HEAD.
/// - `Branch` mode: `git diff <base_branch>...HEAD` — changes since the common ancestor
///   (three-dot diff), which excludes unrelated commits on the base branch.
///
/// # Errors
/// Returns the captured stderr as an `Err` string if the git subprocess exits non-zero
/// or cannot be spawned.
#[tauri::command]
fn get_diff(repo_path: String, mode: DiffMode) -> Result<String, String> {
    let output = match mode {
        DiffMode::WorkingTree => std::process::Command::new("git")
            .args(["-C", &repo_path, "diff", "HEAD"])
            .output()
            .map_err(|e| e.to_string())?,

        DiffMode::Branch { base_branch } => {
            // Three-dot syntax: diff between the merge-base of base_branch and HEAD,
            // so only commits unique to the current branch are included.
            let range = format!("{}...HEAD", base_branch);
            std::process::Command::new("git")
                .args(["-C", &repo_path, "diff", &range])
                .output()
                .map_err(|e| e.to_string())?
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Returns a list of all local branch names for the repository at `repo_path`.
///
/// Uses `--format=%(refname:short)` to get clean branch names without the leading
/// `* ` prefix that `git branch` would otherwise include.
///
/// # Errors
/// Returns the captured stderr as an `Err` string if the git subprocess exits non-zero
/// or cannot be spawned.
#[tauri::command]
fn list_branches(repo_path: String) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["-C", &repo_path, "branch", "--format=%(refname:short)"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches = stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    Ok(branches)
}

/// Returns the name of the currently checked-out branch for the repository at `repo_path`.
///
/// Uses `rev-parse --abbrev-ref HEAD` which returns `"HEAD"` when in a detached HEAD state.
///
/// # Errors
/// Returns the captured stderr as an `Err` string if the git subprocess exits non-zero
/// or cannot be spawned.
#[tauri::command]
fn get_current_branch(repo_path: String) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["-C", &repo_path, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Comment DB helpers ────────────────────────────────────────────────────
// These are plain functions (not Tauri commands) so they can be called from
// integration tests without a running Tauri instance.

fn db_list_comments(
    conn: &rusqlite::Connection,
    repo_id: i64,
) -> Result<Vec<Comment>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, repo_id, file_path, line_num, body \
             FROM comments \
             WHERE repo_id = ?1 \
             ORDER BY file_path, line_num",
        )
        .map_err(|e| e.to_string())?;

    let comments = stmt
        .query_map(rusqlite::params![repo_id], |row| {
            Ok(Comment {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                file_path: row.get(2)?,
                line_num: row.get(3)?,
                body: row.get(4)?,
                is_outdated: false,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(comments)
}

fn db_upsert_comment(
    conn: &rusqlite::Connection,
    repo_id: i64,
    file_path: &str,
    line_num: i64,
    body: &str,
    id: Option<i64>,
) -> Result<Comment, String> {
    let now = chrono::Utc::now().to_rfc3339();

    match id {
        None => {
            // Insert a new comment.
            conn.execute(
                "INSERT INTO comments (repo_id, file_path, line_num, body, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                rusqlite::params![repo_id, file_path, line_num, body, now],
            )
            .map_err(|e| e.to_string())?;

            let new_id = conn.last_insert_rowid();
            Ok(Comment {
                id: new_id,
                repo_id,
                file_path: file_path.to_string(),
                line_num,
                body: body.to_string(),
                is_outdated: false,
            })
        }
        Some(existing_id) => {
            // Update body and updated_at for the existing row.
            let rows_changed = conn
                .execute(
                    "UPDATE comments SET body = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![body, now, existing_id],
                )
                .map_err(|e| e.to_string())?;

            if rows_changed == 0 {
                return Err(format!("Comment {} not found", existing_id));
            }

            // Re-fetch to get the canonical stored values.
            conn.query_row(
                "SELECT id, repo_id, file_path, line_num, body FROM comments WHERE id = ?1",
                rusqlite::params![existing_id],
                |row| {
                    Ok(Comment {
                        id: row.get(0)?,
                        repo_id: row.get(1)?,
                        file_path: row.get(2)?,
                        line_num: row.get(3)?,
                        body: row.get(4)?,
                        is_outdated: false,
                    })
                },
            )
            .map_err(|e| e.to_string())
        }
    }
}

fn db_delete_comment(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM comments WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn db_delete_all_comments(conn: &rusqlite::Connection, repo_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM comments WHERE repo_id = ?1",
        rusqlite::params![repo_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Comment Tauri commands ────────────────────────────────────────────────

/// Returns all comments for `repo_id` ordered by file path then line number.
#[tauri::command]
fn list_comments(repo_id: i64, state: tauri::State<'_, DbState>) -> Result<Vec<Comment>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db_list_comments(&conn, repo_id)
}

/// Creates or updates a comment.
///
/// - `id = None` → insert; returns the new comment with its generated id.
/// - `id = Some(n)` → update `body` and `updated_at` for comment `n`; returns
///   the updated comment.
#[tauri::command]
fn upsert_comment(
    repo_id: i64,
    file_path: String,
    line_num: i64,
    body: String,
    id: Option<i64>,
    state: tauri::State<'_, DbState>,
) -> Result<Comment, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db_upsert_comment(&conn, repo_id, &file_path, line_num, &body, id)
}

/// Deletes the comment with the given `id`.
#[tauri::command]
fn delete_comment(id: i64, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db_delete_comment(&conn, id)
}

/// Deletes all comments belonging to `repo_id`.
#[tauri::command]
fn delete_all_comments(repo_id: i64, state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db_delete_all_comments(&conn, repo_id)
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
        .plugin(tauri_plugin_dialog::init())
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
            remove_repo,
            get_diff,
            list_branches,
            get_current_branch,
            list_comments,
            upsert_comment,
            delete_comment,
            delete_all_comments
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ── Integration tests ─────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// Open an in-memory SQLite database and apply the schema so every test
    /// starts from a clean, fully-migrated state.
    fn new_test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(INITIAL_SCHEMA).unwrap();
        conn
    }

    /// Insert a minimal repo row so foreign-key constraints are satisfied.
    fn seed_repo(conn: &rusqlite::Connection) -> i64 {
        conn.execute(
            "INSERT INTO repos (path, name, added_at) VALUES ('/tmp/test-repo', 'test-repo', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn list_comments_empty_returns_empty_vec() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        let comments = db_list_comments(&conn, repo_id).unwrap();
        assert!(comments.is_empty());
    }

    #[test]
    fn upsert_insert_creates_comment_with_generated_id() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        let comment = db_upsert_comment(&conn, repo_id, "src/main.ts", 42, "body text", None).unwrap();
        assert!(comment.id > 0);
        assert_eq!(comment.repo_id, repo_id);
        assert_eq!(comment.file_path, "src/main.ts");
        assert_eq!(comment.line_num, 42);
        assert_eq!(comment.body, "body text");
        assert!(!comment.is_outdated);
    }

    #[test]
    fn list_comments_returns_inserted_comment() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        db_upsert_comment(&conn, repo_id, "src/main.ts", 42, "body text", None).unwrap();
        let comments = db_list_comments(&conn, repo_id).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].file_path, "src/main.ts");
        assert_eq!(comments[0].line_num, 42);
    }

    #[test]
    fn list_comments_ordered_by_file_then_line() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        db_upsert_comment(&conn, repo_id, "src/b.ts", 10, "b10", None).unwrap();
        db_upsert_comment(&conn, repo_id, "src/a.ts", 20, "a20", None).unwrap();
        db_upsert_comment(&conn, repo_id, "src/a.ts", 5, "a5", None).unwrap();
        let comments = db_list_comments(&conn, repo_id).unwrap();
        assert_eq!(comments[0].file_path, "src/a.ts");
        assert_eq!(comments[0].line_num, 5);
        assert_eq!(comments[1].file_path, "src/a.ts");
        assert_eq!(comments[1].line_num, 20);
        assert_eq!(comments[2].file_path, "src/b.ts");
        assert_eq!(comments[2].line_num, 10);
    }

    #[test]
    fn upsert_update_changes_body_and_preserves_other_fields() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        let original = db_upsert_comment(&conn, repo_id, "src/main.ts", 42, "original", None).unwrap();
        let updated = db_upsert_comment(&conn, repo_id, "src/main.ts", 42, "updated", Some(original.id)).unwrap();
        assert_eq!(updated.id, original.id);
        assert_eq!(updated.body, "updated");
        assert_eq!(updated.line_num, 42);
        assert_eq!(updated.file_path, "src/main.ts");
    }

    #[test]
    fn upsert_update_nonexistent_id_returns_error() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        let result = db_upsert_comment(&conn, repo_id, "src/main.ts", 1, "body", Some(9999));
        assert!(result.is_err());
    }

    #[test]
    fn delete_comment_removes_it_from_list() {
        let conn = new_test_db();
        let repo_id = seed_repo(&conn);
        let comment = db_upsert_comment(&conn, repo_id, "src/main.ts", 42, "body", None).unwrap();
        db_delete_comment(&conn, comment.id).unwrap();
        let comments = db_list_comments(&conn, repo_id).unwrap();
        assert!(comments.is_empty());
    }

    #[test]
    fn delete_all_comments_clears_repo_only() {
        let conn = new_test_db();
        let repo_a = seed_repo(&conn);
        // Insert a second repo.
        conn.execute(
            "INSERT INTO repos (path, name, added_at) VALUES ('/tmp/repo-b', 'repo-b', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let repo_b = conn.last_insert_rowid();

        db_upsert_comment(&conn, repo_a, "src/a.ts", 1, "a comment", None).unwrap();
        db_upsert_comment(&conn, repo_b, "src/b.ts", 2, "b comment", None).unwrap();

        db_delete_all_comments(&conn, repo_a).unwrap();

        assert!(db_list_comments(&conn, repo_a).unwrap().is_empty());
        assert_eq!(db_list_comments(&conn, repo_b).unwrap().len(), 1);
    }
}
