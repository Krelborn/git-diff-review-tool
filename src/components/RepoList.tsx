import { open } from "@tauri-apps/plugin-dialog";
import { type JSX } from "react";

import { useAppStore } from "../store";

/**
 * Sidebar section that lists all tracked repositories and allows the user to
 * add or remove them via the Tauri dialog API.
 */
export function RepoList(): JSX.Element {
  const addRepo = useAppStore((s) => s.addRepo);
  const removeRepo = useAppStore((s) => s.removeRepo);
  const repos = useAppStore((s) => s.repos);
  const reposError = useAppStore((s) => s.reposError);
  const selectedRepoId = useAppStore((s) => s.selectedRepoId);
  const selectRepo = useAppStore((s) => s.selectRepo);

  async function handleAdd(): Promise<void> {
    const selected = await open({ directory: true, multiple: false });

    // `open` returns null when the user cancels the picker.
    if (selected === null) {
      return;
    }

    await addRepo(selected);
  }

  return (
    <div className="repo-list">
      <div className="sidebar-section-header">
        Repositories
        <button
          aria-label="Add repository"
          className="sidebar-add-btn"
          onClick={() => void handleAdd()}
          type="button"
        >
          +
        </button>
      </div>

      {repos.map((repo) => (
        <div
          className={`repo-item${selectedRepoId === repo.id ? " active" : ""}`}
          key={repo.id}
          onClick={() => selectRepo(repo.id)}
        >
          <span className="repo-name" title={repo.path}>
            {repo.name}
          </span>
          <button
            aria-label={`Remove ${repo.name}`}
            className="repo-remove-btn"
            onClick={(e) => {
              // Prevent the click from also triggering selectRepo.
              e.stopPropagation();
              void removeRepo(repo.id);
            }}
            type="button"
          >
            ×
          </button>
        </div>
      ))}

      {reposError !== null && (
        <div className="repo-error" role="alert">
          {reposError}
        </div>
      )}
    </div>
  );
}
