# Requirements: Git Diff Review Tool

## Problem Statement

Developers using AI coding assistants (e.g. Claude Code) need a way to systematically review AI-generated changes, annotate problems, and generate structured prompts to feed back into the AI for correction. The current workflow is manual and error-prone — developers eyeball diffs in a terminal or IDE, mentally track issues, and write prompts from memory. This tool makes that review loop fast and repeatable.

## User Stories

- As a developer, I want to open any local git repo in the app so I can review its changes without leaving my review workflow.
- As a developer, I want to see all changed files and their diffs so I can understand what the AI modified.
- As a developer, I want to annotate specific diff lines with comments so I can flag problems as I find them.
- As a developer, I want my comments saved across sessions so I can pause and resume a review.
- As a developer, I want to switch between "working-tree vs HEAD" and "branch vs base branch" diff modes so I can use the same tool for mid-session review and pre-PR cleanup.
- As a developer, I want to generate a structured issue list copied to clipboard so I can paste it directly into Claude Code.
- As a developer, I want to clear all comments for a repo so I can start a fresh review session.

---

## Functional Requirements

### FR-1: Repository Management

- **Priority:** Must Have
- **Description:** Users can add local git repositories to the app and switch between them.
- **Acceptance Criteria:**
  - [ ] User can add a repo by selecting a folder via OS file picker
  - [ ] Added repos persist across app restarts
  - [ ] User can remove a repo from the list
  - [ ] App validates that selected folder is a git repository before adding

### FR-2: Diff Mode Selection

- **Priority:** Must Have
- **Description:** For each repo, users can choose which diff to view.
- **Acceptance Criteria:**
  - [ ] Mode A: Uncommitted working-tree changes vs HEAD (default)
  - [ ] Mode B: Current branch vs a selected base branch
  - [ ] User can switch modes without losing comments
  - [ ] Branch list is populated from the repo's local branches

### FR-3: File Tree Navigation

- **Priority:** Must Have
- **Description:** Users can see all changed files and navigate between them.
- **Acceptance Criteria:**
  - [ ] Changed files shown in a sidebar tree grouped by directory
  - [ ] File shows change type indicator (modified / added / deleted / renamed)
  - [ ] Files with comments show a badge with comment count
  - [ ] Clicking a file loads its diff in the main panel

### FR-4: Diff Viewer

- **Priority:** Must Have
- **Description:** Users can read diffs in their preferred layout with syntax highlighting.
- **Acceptance Criteria:**
  - [ ] Unified diff view (single pane) with added/removed line highlighting
  - [ ] Split diff view (two panes: old left, new right) with parallel line alignment
  - [ ] Toggle between unified and split view; preference persists across sessions
  - [ ] Line numbers shown for both old and new file in both views
  - [ ] Syntax highlighting based on file extension
  - [ ] Large diffs (> 500 lines) load without freezing the UI

### FR-5: Inline Commenting

- **Priority:** Must Have
- **Description:** Users can add, edit, and delete comments on specific diff lines.
- **Acceptance Criteria:**
  - [ ] Clicking a line number opens a comment input on that line
  - [ ] Comment input supports multi-line plain text
  - [ ] Existing comments shown inline below their target line
  - [ ] User can edit or delete an existing comment
  - [ ] Comments anchor to file path + new-file line number
  - [ ] Comments whose anchored line no longer appears in the current diff are shown with an "Outdated" badge (similar to GitHub PR outdated comments)
  - [ ] Outdated comments are still included in the generated prompt output and can be deleted

### FR-6: Comment Persistence

- **Priority:** Must Have
- **Description:** Comments survive app restarts and are scoped per repo.
- **Acceptance Criteria:**
  - [ ] Comments stored locally (SQLite) keyed by repo path + file path + line number
  - [ ] Comments reload correctly when the same repo is reopened
  - [ ] No data loss on unexpected app close

### FR-7: Generate Prompt

- **Priority:** Must Have
- **Description:** One-click generation of a structured issue list copied to clipboard.
- **Acceptance Criteria:**
  - [ ] "Copy Issues to Clipboard" button visible when the repo has at least one comment
  - [ ] Output is raw issue lines only — no preamble or header text
  - [ ] Output format: one issue per line — `path/to/file.ts:42 — "comment text"`
  - [ ] Issues sorted by file path, then by line number
  - [ ] Success toast confirms copy to clipboard
  - [ ] Button can be scoped to current file or all files (toggle)

### FR-8: Clear Session

- **Priority:** Must Have
- **Description:** Users can delete all comments for a repo to start a new review.
- **Acceptance Criteria:**
  - [ ] "Clear all comments" action available in repo context menu or settings panel
  - [ ] Confirmation dialog shown before deletion
  - [ ] All comments for that repo are deleted; comments for other repos are unaffected

---

## Non-Functional Requirements

### NFR-1: Performance

- Diff for files up to 2,000 lines must render within 500ms
- App launch to usable state within 2 seconds
- No UI jank when scrolling through large diffs

### NFR-2: Platform

- macOS 13 (Ventura) or later
- No Windows or Linux support required at this stage

### NFR-3: Distribution

- Distributable as a signed .app bundle or via direct download (no App Store requirement initially)

### NFR-4: Data Locality

- All data (repos, comments) stored locally. No network access required or made.

---

## Constraints

- Must use Tauri (preferred) or Electrobun as the app shell — not pure Electron
- Frontend in TypeScript + React
- Git operations via the system `git` binary (not a bundled git)

## Dependencies

- System `git` must be installed and on PATH
- macOS Ventura+ WebView (WKWebView via Tauri)

## Out of Scope

- Cloud sync of comments
- Collaborative/multi-user review
- PR creation or GitHub/GitLab API integration
- Comment threads / replies
- AI suggestions inline in the diff

## Decisions Recorded

- Comments whose lines have shifted show an "Outdated" badge; they remain in output and are deletable (v1 approach — stale line indicator rather than silent misalignment)
- Generated prompt is raw issue lines only, no preamble — format kept as generic as possible for paste into any AI tool
- No target Claude Code skill mandated; output format is plain text designed to be universally readable

## Implementation Status

### Phase 0 — Foundation

- [x] **Task 0.1 — Project Scaffold** (2026-03-28): Tauri v2 + Vite + React 19 + TypeScript scaffold created. `tauri-plugin-sql`, `tauri-plugin-shell`, and `tauri-plugin-clipboard-manager` declared in `Cargo.toml` and wired into `capabilities/default.json`. ESLint (flat config, v9) and Prettier configured. TypeScript strict mode verified. `npm run lint` and `tsc --noEmit` pass cleanly.
- [ ] **Task 0.2 — SQLite Schema Migration**: pending (blocked — Rust/Cargo not yet installed)

### Phase 2 — Diff Loading & File Tree

- [x] **Task 2.2 — Frontend: Diff Text Parser** (2026-03-28): `src/types/diff.ts` defines `DiffFile`, `DiffHunk`, `DiffLine`, `Comment`, `Repo`, `DiffMode`, and `DiffLayout`. `src/lib/diffParser.ts` implements `parseDiff(rawDiff: string): DiffFile[]` — splits on `diff --git` boundaries, extracts per-file paths and change type from `---`/`+++` headers, parses `@@` hunk headers for line number tracking, and classifies each line as `added`/`removed`/`context`. Handles new files, deleted files, and renames. TypeScript strict mode verified.
- [x] **Task 2.3 — Frontend: diff Zustand Slice + DiffModeSelector + BranchPicker** (2026-03-28): `src/store/reposSlice.ts` provides minimal `ReposSlice` (repos list, selectedRepoId). `src/store/diffSlice.ts` provides `DiffSlice` with `diffMode`, `fileDiffs`, `selectedFilePath`, `branches`, `currentBranch`, `isDiffLoading`, `diffError`, and actions `setDiffMode`, `loadDiff`, `loadBranches`, `selectFile`. `loadDiff` calls Tauri `get_diff` command and pipes result through `parseDiff`. `loadBranches` calls `list_branches` + `get_current_branch`. `src/store/index.ts` combines slices into `AppStore` via `useAppStore`. `src/components/DiffModeSelector.tsx` renders the working-tree/branch toggle and triggers `loadDiff` on switch to working-tree. `src/components/BranchPicker.tsx` renders branch `<select>` only when mode is "branch", filtering out the current branch; triggers `loadDiff` on selection. `App.tsx` wires a `useEffect` to reload diff + branches when selected repo changes. Tauri commands will no-op until Task 2.1 is implemented. TypeScript strict mode verified.
