import { invoke } from "@tauri-apps/api/core";
import { StateCreator } from "zustand";

import { Repo } from "../types/diff";

export interface ReposSlice {
  repos: Repo[];

  /** Non-null when the most recent add/load operation failed. */
  reposError: string | null;

  selectedRepoId: number | null;

  /** Replace the entire repos list (used for initial hydration). */
  setRepos: (repos: Repo[]) => void;

  /** Mark a repo as selected by id. */
  selectRepo: (repoId: number) => void;

  /** Fetch all repos from the backend and hydrate the slice. */
  loadRepos: () => Promise<void>;

  /**
   * Invoke `add_repo` with the given filesystem path and append the
   * returned Repo to the list. Sets `reposError` on failure.
   */
  addRepo: (path: string) => Promise<void>;

  /**
   * Optimistically remove a repo by id, then invoke `remove_repo`.
   * If the backend call fails the repo is restored and `reposError` is set.
   */
  removeRepo: (id: number) => Promise<void>;
}

export const createReposSlice: StateCreator<ReposSlice, [], [], ReposSlice> = (set, get) => ({
  repos: [],
  reposError: null,
  selectedRepoId: null,

  setRepos: (repos) => set({ repos }),

  selectRepo: (repoId) => set({ selectedRepoId: repoId }),

  loadRepos: async () => {
    const repos = await invoke<Repo[]>("list_repos");
    set({ repos });
  },

  addRepo: async (path) => {
    // Clear any prior error before attempting a new add.
    set({ reposError: null });

    try {
      const repo = await invoke<Repo>("add_repo", { path });
      set((state) => ({ repos: [...state.repos, repo] }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ reposError: message });
    }
  },

  removeRepo: async (id) => {
    // Snapshot current list for rollback if the backend call fails.
    const previous = get().repos;
    set((state) => ({ repos: state.repos.filter((r) => r.id !== id) }));

    try {
      await invoke<void>("remove_repo", { id });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ repos: previous, reposError: message });
    }
  },
});
