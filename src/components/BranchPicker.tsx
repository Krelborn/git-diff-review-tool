import { type JSX, ChangeEvent } from "react";

import { useAppStore } from "../store";

/**
 * Renders a branch selector when the current diff mode is "vs branch".
 * Returns null in any other mode so the caller need not conditionally render it.
 */
export function BranchPicker(): JSX.Element | null {
  const branches = useAppStore((s) => s.branches);
  const currentBranch = useAppStore((s) => s.currentBranch);
  const diffMode = useAppStore((s) => s.diffMode);
  const loadDiff = useAppStore((s) => s.loadDiff);
  const repos = useAppStore((s) => s.repos);
  const selectedRepoId = useAppStore((s) => s.selectedRepoId);
  const setDiffMode = useAppStore((s) => s.setDiffMode);

  if (diffMode.type !== "branch") {
    return null;
  }

  // Exclude the current branch from the list — comparing HEAD against itself
  // is never useful.
  const availableBranches = branches.filter((b) => b !== currentBranch);

  const handleBranchChange = (e: ChangeEvent<HTMLSelectElement>): void => {
    const baseBranch = e.target.value;
    setDiffMode({ type: "branch", baseBranch });
    const repo = repos.find((r) => r.id === selectedRepoId);
    if (repo && baseBranch) {
      void loadDiff(repo.path);
    }
  };

  return (
    <select className="branch-picker" value={diffMode.baseBranch} onChange={handleBranchChange}>
      <option disabled={true} value="">
        Select base branch…
      </option>
      {availableBranches.map((branch) => (
        <option key={branch} value={branch}>
          {branch}
        </option>
      ))}
    </select>
  );
}
