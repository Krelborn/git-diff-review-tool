import { StateCreator } from "zustand";

import { Repo } from "../types/diff";

export interface ReposSlice {
  repos: Repo[];
  selectedRepoId: number | null;
  setRepos: (repos: Repo[]) => void;
  selectRepo: (repoId: number) => void;
}

export const createReposSlice: StateCreator<ReposSlice, [], [], ReposSlice> = (set) => ({
  repos: [],
  selectedRepoId: null,
  setRepos: (repos) => set({ repos }),
  selectRepo: (repoId) => set({ selectedRepoId: repoId }),
});
