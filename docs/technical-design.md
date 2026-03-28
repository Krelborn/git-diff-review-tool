# Technical Design: Git Diff Review Tool

## Summary

A macOS desktop app built with Tauri v2 (Rust backend + React/TypeScript frontend) that reads git diffs from local repositories, renders them with inline commenting, persists comments in a local SQLite database, and produces structured issue lists for pasting into Claude Code.

The Rust backend handles all git and filesystem operations via shelling out to the system `git` binary. The React frontend owns all UI state and renders diffs as parsed structured data. Comments are persisted via Tauri's SQLite plugin.

---

## Goals

- Fast, native-feeling macOS app with minimal dependencies
- Clean separation: Rust layer for system IO, React layer for all UI
- Offline-only — no network calls
- Easy to extend with new diff modes or output formats

## Non-Goals

- Cross-platform support
- Remote git operations (fetch, push)
- GitHub/GitLab API integration
- Split diff alignment for binary or very large files (may fall back to unified)

---

## Architecture

### System Diagram

```
┌─────────────────────────────────────────────────┐
│                 Tauri App Shell                  │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │           React Frontend (WKWebView)      │   │
│  │                                           │   │
│  │  RepoSidebar  │  DiffViewer  │  CommentPanel │ │
│  │               │              │              │ │
│  │         Zustand Store                     │  │
│  └──────────────┬───────────────────────────┘   │
│                 │  invoke() / listen()            │
│  ┌──────────────▼───────────────────────────┐   │
│  │           Rust Commands                   │   │
│  │                                           │   │
│  │  git_diff()   list_repos()   git_branches()│  │
│  │                    │                      │   │
│  │              shells out to `git`          │   │
│  └──────────────┬───────────────────────────┘   │
│                 │                                 │
│  ┌──────────────▼───────────────────────────┐   │
│  │  tauri-plugin-sql (SQLite)               │   │
│  │  ~/.local/share/review-tool/comments.db  │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Key Decisions

| Decision       | Options Considered                                         | Choice                          | Rationale                                                                                                                                              |
| -------------- | ---------------------------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| App framework  | Tauri v2, Electrobun, Electron                             | **Tauri v2**                    | Most mature, strong macOS support, active community, good SQLite plugin ecosystem. Electrobun is too early-stage for production use.                   |
| Git access     | `git2` Rust crate (libgit2), shell out to `git` CLI        | **Shell out to `git`**          | Simpler, always consistent with user's actual git version, no extra Rust dep to maintain. Sufficient for read-only diff operations.                    |
| Storage        | SQLite via tauri-plugin-sql, JSON file, tauri-plugin-store | **SQLite via tauri-plugin-sql** | Comments are relational data (scoped by repo + file + line), SQLite gives easy querying without manual JSON management.                                |
| Diff parsing   | Server-side (Rust parses unified diff), client-side        | **Client-side**                 | Rust sends raw unified diff text; React parses it into structured hunks. Keeps Rust layer thin and logic testable in TypeScript.                       |
| Frontend state | Redux, Zustand, Jotai                                      | **Zustand**                     | Lightweight, minimal boilerplate, good fit for flat app state.                                                                                         |
| Diff rendering | Monaco Editor, CodeMirror, custom                          | **Custom renderer**             | Monaco is very heavy (~2MB) for read-only diff display. A custom component over the parsed hunk data gives full control over comment injection points. |

---

## Data Model

### SQLite Schema

```sql
-- Persisted repos list
CREATE TABLE repos (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  path     TEXT NOT NULL UNIQUE,   -- absolute path to repo root
  name     TEXT NOT NULL,          -- display name (last path segment)
  added_at TEXT NOT NULL           -- ISO 8601
);

-- Comments anchored to a specific line in a diff
CREATE TABLE comments (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id    INTEGER NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
  file_path  TEXT NOT NULL,   -- repo-relative path, e.g. "src/foo.ts"
  line_num   INTEGER NOT NULL, -- new-file line number in the diff
  body       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_comments_repo ON comments(repo_id);
CREATE INDEX idx_comments_file ON comments(repo_id, file_path);
```

### TypeScript Types (Frontend)

```typescript
interface Repo {
  id: number;
  path: string;
  name: string;
}

type DiffMode =
  | { type: "working-tree" } // working tree vs HEAD
  | { type: "branch"; baseBranch: string }; // current branch vs base

interface DiffFile {
  path: string; // repo-relative path
  oldPath?: string; // for renames
  changeType: "modified" | "added" | "deleted" | "renamed";
  hunks: DiffHunk[];
}

interface DiffHunk {
  header: string; // "@@ -1,7 +1,9 @@"
  lines: DiffLine[];
}

interface DiffLine {
  type: "context" | "added" | "removed";
  oldLineNum?: number;
  newLineNum?: number;
  content: string; // line text without the leading +/-/
}

interface Comment {
  id: number;
  repoId: number;
  filePath: string;
  lineNum: number;
  body: string;
  isOutdated: boolean; // true when lineNum no longer appears in the current diff
}

type DiffLayout = "unified" | "split";
```

---

## Tauri Commands (Rust → Frontend API)

```
list_repos()                           → Repo[]
add_repo(path: string)                 → Repo
remove_repo(id: number)                → void

get_diff(repo_id, mode: DiffMode)      → DiffFile[]
list_branches(repo_id)                 → string[]
get_current_branch(repo_id)            → string

list_comments(repo_id)                 → Comment[]
upsert_comment(repo_id, file, line, body) → Comment
delete_comment(id)                     → void
delete_all_comments(repo_id)           → void
```

All commands communicate via Tauri's `invoke()` bridge. Errors surface as rejected promises with a string message.

---

## Frontend Component Architecture

```
App
├── Sidebar
│   ├── RepoList
│   │   └── RepoItem (+ add/remove controls)
│   └── FileTree
│       └── FileItem (change badge, comment count badge)
├── MainPanel
│   ├── DiffToolbar
│   │   ├── DiffModeSelector (working-tree | branch)
│   │   ├── BranchPicker (shown in branch mode)
│   │   ├── DiffLayoutToggle (unified | split)
│   │   └── CopyIssuesButton
│   └── DiffViewer  ← renders either UnifiedDiffView or SplitDiffView
│       ├── UnifiedDiffView
│       │   └── DiffHunk[]
│       │       └── DiffLine (context | added | removed)
│       │           ├── LineGutter (old + new line nums, click to comment)
│       │           ├── LineContent
│       │           └── CommentThread (below annotated lines)
│       └── SplitDiffView
│           └── DiffHunk[]
│               └── SplitDiffRow (aligned old/new line pair)
│                   ├── OldLineSide (line num + removed content)
│                   ├── NewLineSide (line num + added content, click to comment)
│                   └── CommentThread (spans full width below row)
└── Toaster (clipboard copy confirmation)
```

---

## State Management

### Zustand Store Slices

| Slice      | State                                                               | Notes                                         |
| ---------- | ------------------------------------------------------------------- | --------------------------------------------- |
| `repos`    | `Repo[]`, `activeRepoId`                                            | Loaded from DB on mount                       |
| `diff`     | `DiffFile[]`, `activeDiffMode`, `baseBranch`, `activeFilePath`      | Refreshed on repo/mode change                 |
| `comments` | `Comment[]`                                                         | Loaded per repo; invalidated on upsert/delete |
| `ui`       | `commentDraftLineNum`, `commentDraftBody`, `diffLayout: DiffLayout` | `diffLayout` persisted to localStorage        |

---

## Diff Parsing

The Rust command runs `git diff` and returns the raw unified diff string. The frontend parses this using a small TypeScript parser:

1. Split on `diff --git` headers to extract per-file sections
2. Parse `--- a/...` / `+++ b/...` lines for file paths and change type
3. Split into hunks on `@@` lines
4. Parse each line: prefix `+` → added, `-` → removed, ` ` → context
5. Track `newLineNum` counter per hunk starting from the `+start` value in the `@@` header

This produces the `DiffFile[]` structure the frontend renders directly.

### Outdated Comment Detection

After each diff load, the frontend computes the set of `newLineNum` values present in the current diff for each file. Any stored comment whose `(filePath, lineNum)` pair is not in that set is marked `isOutdated: true` in the Zustand store (not in the DB — this is computed on the fly). Outdated comments render with a yellow "Outdated" badge above the comment body and are grouped at the bottom of the file's comment list rather than inline.

### Split Diff Line Pairing

Split view requires aligning removed lines (old side) with added lines (new side). Algorithm per hunk:

1. Collect removed lines and added lines separately
2. Pair them positionally (removed[0] ↔ added[0], etc.)
3. If one side has more lines, the shorter side gets empty filler rows
4. Context lines span both columns

---

## Generated Prompt Format

```
src/auth/middleware.ts:42 — "This mutates the request object in place; should return a new object"
src/auth/middleware.ts:87 — "Missing error handling for expired tokens"
src/components/UserCard.tsx:15 — "Props interface should be exported for reuse"
```

- Raw issue lines only — no header, no preamble
- One line per comment, sorted by file path then line number
- Outdated comments included with their last-known line number (unchanged)
- Written to clipboard via Tauri's clipboard plugin
- Scope toggle: all files vs current file only

---

## Error Handling Strategy

| Error                                  | Handling                                            |
| -------------------------------------- | --------------------------------------------------- |
| Selected folder is not a git repo      | Validation in `add_repo` command; error toast in UI |
| `git` not found on PATH                | Error toast with instructions to install git        |
| `git diff` fails (detached HEAD, etc.) | Error state in diff panel with raw error message    |
| DB write fails                         | Console error + toast; no silent data loss          |

---

## Performance Considerations

- Diffs are computed on demand (not precomputed) — acceptable since git is fast for local repos
- For files > 500 lines of diff, implement virtual scrolling in `DiffViewer` using `@tanstack/react-virtual`
- Comments keyed by `Map<filePath, Map<lineNum, Comment>>` in the store for O(1) lookup during rendering
- Branch list fetched once per repo selection, not on every render

---

## Security Considerations

- Repo paths are passed to `git` CLI via Rust's `Command::arg()` (not shell string interpolation) — no shell injection
- No network access; no external data loaded
- Comment bodies are plain text rendered in React — no HTML injection risk if rendered as `{text}` not `dangerouslySetInnerHTML`

---

## Risks and Mitigations

| Risk                                         | Likelihood | Impact | Mitigation                                                                                                             |
| -------------------------------------------- | ---------- | ------ | ---------------------------------------------------------------------------------------------------------------------- |
| Line number anchoring breaks after rebase    | High       | Med    | "Outdated" badge shown when comment's line no longer appears in current diff; comment preserved and included in output |
| Tauri WKWebView CSS inconsistencies on macOS | Med        | Low    | Test on target macOS versions early; avoid CSS that differs between Chrome and Safari                                  |
| Large monorepo diffs (10k+ lines) cause jank | Med        | Med    | Virtual scroll in DiffViewer; truncate files > 1000 diff lines with "show more"                                        |
| Electrobun matures and becomes preferable    | Low        | Low    | Tauri abstraction is thin — migration path exists if needed                                                            |

---

## Decisions Recorded

- Outdated comments: shown with yellow "Outdated" badge, not deleted, included in generated output at last-known line number
- Generated prompt: raw issue lines only, no preamble — format kept generic for paste into any AI tool
- Split diff layout preference: persisted to localStorage (not SQLite) — it's a UI preference, not review data
