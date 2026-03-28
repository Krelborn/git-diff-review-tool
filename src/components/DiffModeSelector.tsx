import { type JSX } from "react";

import { DiffMode } from "../types/diff";
import { useAppStore } from "../store";

/**
 * Toggle between "working tree" and "vs branch" diff modes.
 * When switching to working-tree, immediately triggers a diff load
 * if a repo is already selected.
 */
export function DiffModeSelector(): JSX.Element {
  const diffMode = useAppStore((s) => s.diffMode);
  const loadDiff = useAppStore((s) => s.loadDiff);
  const repos = useAppStore((s) => s.repos);
  const selectedRepoId = useAppStore((s) => s.selectedRepoId);
  const setDiffMode = useAppStore((s) => s.setDiffMode);

  const handleModeChange = (type: DiffMode["type"]): void => {
    if (type === "working-tree") {
      setDiffMode({ type: "working-tree" });
      const repo = repos.find((r) => r.id === selectedRepoId);
      if (repo) {
        void loadDiff(repo.path);
      }
    } else {
      // Don't trigger a load yet — BranchPicker will do so once a branch is chosen
      setDiffMode({ type: "branch", baseBranch: "" });
    }
  };

  return (
    <div className="diff-mode-toggle">
      <button
        className={`diff-mode-btn${diffMode.type === "working-tree" ? " active" : ""}`}
        onClick={() => handleModeChange("working-tree")}
      >
        Working tree
      </button>
      <button
        className={`diff-mode-btn${diffMode.type === "branch" ? " active" : ""}`}
        onClick={() => handleModeChange("branch")}
      >
        vs branch
      </button>
    </div>
  );
}
