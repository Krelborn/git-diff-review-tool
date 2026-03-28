import { create } from "zustand";

import { createDiffSlice, DiffSlice } from "./diffSlice";
import { createReposSlice, ReposSlice } from "./reposSlice";

export type AppStore = DiffSlice & ReposSlice;

export const useAppStore = create<AppStore>()((...a) => ({
  ...createReposSlice(...a),
  ...createDiffSlice(...a),
}));
