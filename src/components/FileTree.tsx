import { type JSX, KeyboardEvent, useCallback, useMemo } from "react";

import { DiffFile } from "../types/diff";
import { useAppStore } from "../store";

/** Maps each change type to its badge colour, mirroring the global design tokens. */
const BADGE_COLORS: Record<DiffFile["changeType"], string> = {
  added: "#73c991",
  deleted: "#f47174",
  modified: "#e8a838",
  renamed: "#4c8ef7",
};

/**
 * A single entry within a directory group, pairing a DiffFile with the
 * bare filename that should be displayed in the tree.
 */
interface FileEntry {
  /** The bare filename component, e.g. "diff.ts". */
  filename: string;

  /** The full DiffFile record from the store. */
  file: DiffFile;
}

/**
 * A directory bucket produced by the grouping pass.
 */
interface DirGroup {
  /**
   * The directory path relative to the repo root, e.g. "src/types".
   * The empty string represents the root level.
   */
  dir: string;

  /** Files that live directly inside this directory. */
  entries: FileEntry[];
}

/**
 * Groups an array of DiffFiles by their immediate parent directory and sorts
 * the resulting groups: root ("") first, then all others alphabetically.
 * Within each group files are sorted by filename.
 */
function groupByDirectory(files: DiffFile[]): DirGroup[] {
  const map = new Map<string, FileEntry[]>();

  for (const file of files) {
    const lastSlash = file.path.lastIndexOf("/");
    const dir = lastSlash === -1 ? "" : file.path.slice(0, lastSlash);
    const filename = lastSlash === -1 ? file.path : file.path.slice(lastSlash + 1);

    const bucket = map.get(dir);
    if (bucket !== undefined) {
      bucket.push({ file, filename });
    } else {
      map.set(dir, [{ file, filename }]);
    }
  }

  // Sort entries within each bucket by filename
  for (const entries of map.values()) {
    entries.sort((a, b) => a.filename.localeCompare(b.filename));
  }

  // Build sorted group array: root first, then remaining groups alphabetically
  const dirs = [...map.keys()].sort((a, b) => {
    if (a === "") {
      return -1;
    }
    if (b === "") {
      return 1;
    }
    return a.localeCompare(b);
  });

  return dirs.map((dir) => ({
    dir,
    // Non-null assertion is safe here: we only iterate keys that exist in the map
    entries: map.get(dir) as FileEntry[],
  }));
}

export interface FileTreeProps {
  /**
   * Optional mapping of file path → number of comments attached to that file.
   * When absent, all comment counts default to 0 and no badge is rendered.
   */
  commentCounts?: Map<string, number>;
}

/**
 * Sidebar file tree that displays the current diff's changed files grouped by
 * directory. Selecting a file updates the store's `selectedFilePath`.
 *
 * @param props.commentCounts - Optional per-file comment counts.
 */
export function FileTree({ commentCounts }: FileTreeProps): JSX.Element {
  const fileDiffs = useAppStore((s) => s.fileDiffs);
  const selectFile = useAppStore((s) => s.selectFile);
  const selectedFilePath = useAppStore((s) => s.selectedFilePath);

  const groups = useMemo(() => groupByDirectory(fileDiffs), [fileDiffs]);

  const handleKeyDown = useCallback(
    (path: string) =>
      (e: KeyboardEvent<HTMLDivElement>): void => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          selectFile(path);
        }
      },
    [selectFile]
  );

  if (fileDiffs.length === 0) {
    return <div style={{ color: "var(--text-muted)" }}>No changes</div>;
  }

  return (
    <div className="file-tree">
      {groups.map(({ dir, entries }) => (
        <div key={dir === "" ? "__root__" : dir}>
          {dir !== "" && <div className="dir-label">{dir + "/"}</div>}
          {entries.map(({ file, filename }) => {
            const commentCount = commentCounts?.get(file.path) ?? 0;
            const isActive = file.path === selectedFilePath;

            return (
              <div
                className={`file-item${isActive ? " active" : ""}`}
                key={file.path}
                role="button"
                tabIndex={0}
                onClick={() => selectFile(file.path)}
                onKeyDown={handleKeyDown(file.path)}
              >
                <div
                  className="file-change-badge"
                  style={{ background: BADGE_COLORS[file.changeType] }}
                />
                <span className="file-name">{filename}</span>
                {commentCount > 0 && <span className="file-comment-count">{commentCount}</span>}
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}
