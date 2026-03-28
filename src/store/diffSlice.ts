import { invoke } from "@tauri-apps/api/core";
import { StateCreator } from "zustand";

import { parseDiff } from "../lib/diffParser";
import { DiffFile, DiffMode } from "../types/diff";

export interface DiffSlice {
  branches: string[];
  currentBranch: string;
  diffError: string | null;
  diffMode: DiffMode;
  fileDiffs: DiffFile[];
  isDiffLoading: boolean;
  selectedFilePath: string | null;
  loadBranches: (repoPath: string) => Promise<void>;
  loadDiff: (repoPath: string) => Promise<void>;
  selectFile: (path: string) => void;
  setDiffMode: (mode: DiffMode) => void;
}

export const createDiffSlice: StateCreator<DiffSlice, [], [], DiffSlice> = (set, get) => ({
  branches: [],
  currentBranch: "",
  diffError: null,
  diffMode: { type: "working-tree" },
  fileDiffs: [],
  isDiffLoading: false,
  selectedFilePath: null,

  loadBranches: async (repoPath) => {
    try {
      const [branches, currentBranch] = await Promise.all([
        invoke<string[]>("list_branches", { repoPath }),
        invoke<string>("get_current_branch", { repoPath }),
      ]);
      set({ branches, currentBranch });
    } catch {
      // Branches will be empty until Rust commands are available
    }
  },

  loadDiff: async (repoPath) => {
    set({ isDiffLoading: true, diffError: null });
    try {
      const rawDiff = await invoke<string>("get_diff", {
        repoPath,
        mode: get().diffMode,
      });
      const fileDiffs = parseDiff(rawDiff);
      set({ fileDiffs, isDiffLoading: false });
    } catch (err) {
      set({ diffError: String(err), isDiffLoading: false });
    }
  },

  selectFile: (path) => set({ selectedFilePath: path }),

  setDiffMode: (mode) => set({ diffMode: mode }),
});
