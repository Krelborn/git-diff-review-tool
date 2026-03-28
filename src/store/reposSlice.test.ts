import { create } from "zustand";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Repo } from "../types/diff";
import { createReposSlice, ReposSlice } from "./reposSlice";

// ---------------------------------------------------------------------------
// Mock Tauri's invoke so tests never call into the native runtime.
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  invoke: vi.fn<any>(),
}));

// Import the mocked module so we can manipulate the mock in each test.
import * as tauriCore from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeStore() {
  return create<ReposSlice>()((...a) => createReposSlice(...a));
}

const REPO_A: Repo = { id: 1, name: "alpha", path: "/repos/alpha" };
const REPO_B: Repo = { id: 2, name: "beta", path: "/repos/beta" };

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("reposSlice", () => {
  // Typed reference to the mock — cast once here so every test stays clean.
  const invoke = tauriCore.invoke as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invoke.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("loadRepos", () => {
    it("populates repos from the mocked invoke result", async () => {
      const store = makeStore();
      invoke.mockResolvedValueOnce([REPO_A, REPO_B]);

      await store.getState().loadRepos();

      expect(invoke).toHaveBeenCalledWith("list_repos");
      expect(store.getState().repos).toEqual([REPO_A, REPO_B]);
    });
  });

  describe("addRepo", () => {
    it("appends the returned repo to the list", async () => {
      const store = makeStore();
      store.setState({ repos: [REPO_A] });
      invoke.mockResolvedValueOnce(REPO_B);

      await store.getState().addRepo(REPO_B.path);

      expect(invoke).toHaveBeenCalledWith("add_repo", { path: REPO_B.path });
      expect(store.getState().repos).toEqual([REPO_A, REPO_B]);
    });

    it("sets reposError when invoke throws", async () => {
      const store = makeStore();
      invoke.mockRejectedValueOnce(new Error("permission denied"));

      await store.getState().addRepo("/bad/path");

      expect(store.getState().repos).toEqual([]);
      expect(store.getState().reposError).toBe("permission denied");
    });

    it("clears a prior reposError before attempting the add", async () => {
      const store = makeStore();
      store.setState({ reposError: "stale error" });
      invoke.mockResolvedValueOnce(REPO_A);

      await store.getState().addRepo(REPO_A.path);

      expect(store.getState().reposError).toBeNull();
    });
  });

  describe("removeRepo", () => {
    it("removes the repo with the given id", async () => {
      const store = makeStore();
      store.setState({ repos: [REPO_A, REPO_B] });
      invoke.mockResolvedValueOnce(undefined);

      await store.getState().removeRepo(REPO_A.id);

      expect(invoke).toHaveBeenCalledWith("remove_repo", { id: REPO_A.id });
      expect(store.getState().repos).toEqual([REPO_B]);
    });

    it("restores the list and sets reposError when invoke throws", async () => {
      const store = makeStore();
      store.setState({ repos: [REPO_A, REPO_B] });
      invoke.mockRejectedValueOnce(new Error("not found"));

      await store.getState().removeRepo(REPO_A.id);

      // Optimistic removal is rolled back.
      expect(store.getState().repos).toEqual([REPO_A, REPO_B]);
      expect(store.getState().reposError).toBe("not found");
    });
  });
});
