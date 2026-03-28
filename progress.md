# Delivery Progress

Track issue status here. Update `[ ]` → `[x]` when an issue is closed, or re-fetch with:

```
gh issue list --repo Krelborn/git-diff-review-tool --state all --json number,title,state
```

---

## Phase 0 — Foundation

- [x] [#1 Task 0.1 — Project Scaffold](https://github.com/Krelborn/git-diff-review-tool/issues/1)
- [ ] [#2 Task 0.2 — SQLite Schema Migration](https://github.com/Krelborn/git-diff-review-tool/issues/2)

## Phase 1 — Repository Management

- [ ] [#3 Task 1.1 — Rust: Repo Commands](https://github.com/Krelborn/git-diff-review-tool/issues/3)
- [ ] [#4 Task 1.2 — Frontend: Repo List UI + Zustand repos Slice](https://github.com/Krelborn/git-diff-review-tool/issues/4)

## Phase 2 — Diff Loading & File Tree

- [ ] [#5 Task 2.1 — Rust: Diff + Branch Commands](https://github.com/Krelborn/git-diff-review-tool/issues/5)
- [ ] [#6 Task 2.2 — Frontend: Diff Text Parser](https://github.com/Krelborn/git-diff-review-tool/issues/6)
- [ ] [#7 Task 2.3 — Frontend: diff Zustand Slice + DiffModeSelector + BranchPicker](https://github.com/Krelborn/git-diff-review-tool/issues/7)
- [ ] [#8 Task 2.4 — Frontend: File Tree Sidebar](https://github.com/Krelborn/git-diff-review-tool/issues/8)

## Phase 3 — Unified Diff Viewer

- [ ] [#9 Task 3.1 — Frontend: Unified Diff Renderer](https://github.com/Krelborn/git-diff-review-tool/issues/9)
- [ ] [#10 Task 3.2 — Frontend: Virtual Scroll for Unified View](https://github.com/Krelborn/git-diff-review-tool/issues/10)
- [ ] [#11 Task 3.3 — Frontend: Syntax Highlighting](https://github.com/Krelborn/git-diff-review-tool/issues/11)

## Phase 4 — Inline Commenting

- [ ] [#12 Task 4.1 — Rust: Comment Commands](https://github.com/Krelborn/git-diff-review-tool/issues/12)
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

**1 / 21 complete**

---

## Notes for next session

**Task 0.1 completed (2026-03-28).** Scaffold is in place with all plugins declared. Rust/Cargo is not yet installed on this machine — next task (0.2, SQLite schema migration) requires Cargo to be installed first. The frontend scaffold (`npm run lint`, `tsc --noEmit`) is fully verified. Once Rust is installed, run `cargo tauri dev` to confirm the window launches before starting 0.2.
