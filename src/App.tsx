import { type JSX, useEffect } from "react";

import { BranchPicker } from "./components/BranchPicker";
import { DiffModeSelector } from "./components/DiffModeSelector";
import { useAppStore } from "./store";

function App(): JSX.Element {
  const loadBranches = useAppStore((s) => s.loadBranches);
  const loadDiff = useAppStore((s) => s.loadDiff);
  const repos = useAppStore((s) => s.repos);
  const selectedRepoId = useAppStore((s) => s.selectedRepoId);

  useEffect(() => {
    const repo = repos.find((r) => r.id === selectedRepoId);
    if (repo) {
      void loadDiff(repo.path);
      void loadBranches(repo.path);
    }
    // Re-run whenever the selected repo changes
  }, [loadBranches, loadDiff, repos, selectedRepoId]);

  return (
    <main style={{ fontFamily: "sans-serif", padding: "2rem" }}>
      <h1>Review Tool</h1>
      <div style={{ alignItems: "center", display: "flex", gap: "8px", marginTop: "1rem" }}>
        <DiffModeSelector />
        <BranchPicker />
      </div>
    </main>
  );
}

export default App;
