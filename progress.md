# Delivery Progress

Track issue status here. Update `[ ]` → `[x]` when an issue is closed, or re-fetch with:

```
gh issue list --repo Krelborn/git-diff-review-tool --state all --json number,title,state
```

---

## Phase 0 — Foundation

- [x] [#1 Task 0.1 — Project Scaffold](https://github.com/Krelborn/git-diff-review-tool/issues/1)
- [x] [#2 Task 0.2 — SQLite Schema Migration](https://github.com/Krelborn/git-diff-review-tool/issues/2)

## Phase 1 — Repository Management

- [x] [#3 Task 1.1 — Rust: Repo Commands](https://github.com/Krelborn/git-diff-review-tool/issues/3)
- [x] [#4 Task 1.2 — Frontend: Repo List UI + Zustand repos Slice](https://github.com/Krelborn/git-diff-review-tool/issues/4)

## Phase 2 — Diff Loading & File Tree

- [x] [#5 Task 2.1 — Rust: Diff + Branch Commands](https://github.com/Krelborn/git-diff-review-tool/issues/5)
- [x] [#6 Task 2.2 — Frontend: Diff Text Parser](https://github.com/Krelborn/git-diff-review-tool/issues/6)
- [x] [#7 Task 2.3 — Frontend: diff Zustand Slice + DiffModeSelector + BranchPicker](https://github.com/Krelborn/git-diff-review-tool/issues/7)
- [x] [#8 Task 2.4 — Frontend: File Tree Sidebar](https://github.com/Krelborn/git-diff-review-tool/issues/8)

## Phase 3 — Unified Diff Viewer

- [x] [#9 Task 3.1 — Frontend: Unified Diff Renderer](https://github.com/Krelborn/git-diff-review-tool/issues/9)
- [ ] [#10 Task 3.2 — Frontend: Virtual Scroll for Unified View](https://github.com/Krelborn/git-diff-review-tool/issues/10)
- [ ] [#11 Task 3.3 — Frontend: Syntax Highlighting](https://github.com/Krelborn/git-diff-review-tool/issues/11)

## Phase 4 — Inline Commenting

- [x] [#12 Task 4.1 — Rust: Comment Commands](https://github.com/Krelborn/git-diff-review-tool/issues/12)
- [ ] [#13 Task 4.2 — Frontend: comments Zustand Slice + Outdated Detection](https://github.com/Krelborn/git-diff-review-tool/issues/13)
- [ ] [#14 Task 4.3 — Frontend: Inline Comment Thread UI](https://github.com/Krelborn/git-diff-review-tool/issues/14)

## Phase 5 — Split Diff View

- [ ] [#15 Task 5.1 — Frontend: Split Diff Renderer + Pairing Algorithm](https://github.com/Krelborn/git-diff-review-tool/issues/15)
- [ ] [#16 Task 5.2 — Frontend: DiffLayoutToggle + Layout Persistence](https://github.com/Krelborn/git-diff-review-tool/issues/16)

## Phase 6 — Generate Prompt

- [ ] [#17 Task 6.1 — Frontend: CopyIssuesButton + Scope Toggle + Toast](https://github.com/Krelborn/git-diff-review-tool/issues/17)

## Phase 7 — Clear Session

- [ ] [#18 Task 7.1 — Frontend: Clear Session Button + Inline Confirmation](https://github.com/Krelborn/git-diff-review-tool/issues/18)

## Phase 8 — Polish & Quality

- [ ] [#19 Task 8.1 — Visual Theme: Dark Mode CSS Design System](https://github.com/Krelborn/git-diff-review-tool/issues/19)
- [ ] [#20 Task 8.2 — Error Handling & Empty States](https://github.com/Krelborn/git-diff-review-tool/issues/20)
- [ ] [#21 Task 8.3 — End-to-End Smoke Test Suite](https://github.com/Krelborn/git-diff-review-tool/issues/21)

---

**10 / 21 complete** (#1, #2, #3, #4, #5, #6, #7, #8, #9, #12)

---

## Notes for next session

**Task 0.1 completed (2026-03-28).** Scaffold is in place with all plugins declared. Rust/Cargo is not yet installed on this machine — tasks 0.2, 1.1, 2.1, and 4.1 require Cargo to be installed before they can be implemented or verified.

**Task 2.2 completed (2026-03-28).** `src/types/diff.ts` and `src/lib/diffParser.ts` are in place. The parser takes a raw `git diff` string and returns `DiffFile[]`. All TypeScript types for the frontend are now defined here. Tasks 2.3, 2.4, 3.1, and beyond can proceed using these types and the parser.

**Task 2.3 completed (2026-03-28).** `src/store/` now has `reposSlice.ts`, `diffSlice.ts`, and `index.ts` (`useAppStore`). `DiffModeSelector` and `BranchPicker` components are in `src/components/`. The diff slice calls Tauri commands `get_diff`, `list_branches`, and `get_current_branch` — these silently fail (caught errors exposed via `diffError` state) until Task 2.1 provides the Rust implementations.

**Task 2.4 completed (2026-03-28).** `src/components/FileTree.tsx` groups changed files by directory in the sidebar. Files with comments show a numeric badge (defaults to 0 until Task 4.2 provides the comments slice — accepts optional `commentCounts?: Map<string, number>` prop for reactivity). `App.tsx` now shows a 220px sidebar with the FileTree alongside the main panel.

**Task 3.1 completed (2026-03-28).** `src/components/UnifiedDiffView.tsx` renders DiffFile hunks with the 4-column grid (old line#, new line#, indicator, content). Added=green, removed=red, context=muted. Hunk headers render as distinct separator rows. Empty state shows "Select a file to review". `src/App.css` fully replaced with dark-theme design tokens and CSS classes for FileTree + diff viewer. `App.tsx` wired with toolbar above diff viewer.

**Task 0.2 completed (2026-03-28).** SQLite migration wired via `tauri-plugin-sql` in `src-tauri/src/lib.rs`. Migration v1 creates `repos` and `comments` tables idempotently. `db_health_check()` command returns `"ok"`. Version tracking handled internally by the plugin.

**Task 1.1 completed (2026-03-28).** `list_repos`, `add_repo`, and `remove_repo` Tauri commands are live in `src-tauri/src/lib.rs`. `rusqlite 0.31` and `chrono 0.4` added to `Cargo.toml`. The `.setup()` hook opens the same SQLite file as `tauri-plugin-sql` and bootstraps the schema idempotently via `execute_batch(INITIAL_SCHEMA)`. `add_repo` shells out to `git rev-parse --is-inside-work-tree` for validation; UNIQUE constraint violations map to "Repository already added". `remove_repo` ON DELETE CASCADE cleans up associated comments. `cargo check` clean.

**Task 2.1 completed (2026-03-28).** `get_diff`, `list_branches`, and `get_current_branch` Tauri commands are live in `src-tauri/src/lib.rs`. `DiffMode` Rust enum added with `#[serde(tag = "type", rename_all = "camelCase")]` matching the TypeScript type. `get_diff` calls `git diff HEAD` (working-tree) or `git diff <base>...HEAD` (branch). `list_branches` uses `--format=%(refname:short)`. `get_current_branch` uses `rev-parse --abbrev-ref HEAD`. All three registered in `invoke_handler!`. The diff slice's `loadDiff` and `loadBranches` will now receive real data instead of silently failing.

**Task 1.2 completed (2026-03-28).** `reposSlice` expanded with `loadRepos`, `addRepo` (with error state), and `removeRepo` (optimistic with rollback). `src/components/RepoList.tsx` renders the sidebar repo section with OS folder picker via `@tauri-apps/plugin-dialog`, active-state highlighting, and per-item remove button. `App.tsx` calls `loadRepos()` on mount. 6 Vitest unit tests in `src/store/reposSlice.test.ts` — all pass. Vitest + jsdom added to devDependencies. `tsc --noEmit` clean.

**Task 4.1 completed (2026-03-28).** `list_comments`, `upsert_comment`, `delete_comment`, and `delete_all_comments` Tauri commands are live in `src-tauri/src/lib.rs`. `Comment` struct serialises with camelCase field names; `isOutdated` is always `false` from the backend (computed on the frontend after diffing). DB logic in `db_*()` helper functions so it's testable without Tauri; 8 integration tests using an in-memory SQLite DB, all pass. All four commands registered in `invoke_handler!`.

**Recommended next steps:** Task 4.2 (Frontend: comments Zustand Slice + Outdated Detection) is now unblocked and is the next highest priority — it wires the frontend to the new Rust commands and implements outdated detection logic. Task 4.3 (Inline Comment Thread UI) depends on 4.2.
