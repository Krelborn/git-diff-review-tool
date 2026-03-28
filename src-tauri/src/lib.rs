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

#[tauri::command]
fn db_health_check() -> &'static str {
    "ok"
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
        .invoke_handler(tauri::generate_handler![db_health_check])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
