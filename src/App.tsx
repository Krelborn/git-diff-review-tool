import { type JSX, useEffect } from "react";

import { BranchPicker } from "./components/BranchPicker";
import { DiffModeSelector } from "./components/DiffModeSelector";
import { FileTree } from "./components/FileTree";
import { UnifiedDiffView } from "./components/UnifiedDiffView";
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
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      <aside
        style={{
          borderRight: "1px solid var(--border)",
          display: "flex",
          flexDirection: "column",
          flexShrink: 0,
          overflow: "hidden",
          width: "220px",
        }}
      >
        <FileTree />
      </aside>

      <main
        style={{
          display: "flex",
          flex: 1,
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            alignItems: "center",
            borderBottom: "1px solid var(--border)",
            display: "flex",
            flexShrink: 0,
            gap: "8px",
            padding: "8px 12px",
          }}
        >
          <DiffModeSelector />
          <BranchPicker />
        </div>
        <UnifiedDiffView />
      </main>
    </div>
  );
}

export default App;
