export interface Repo {
  id: number;
  path: string;
  name: string;
}

export type DiffMode = { type: "working-tree" } | { type: "branch"; baseBranch: string };

export interface DiffFile {
  path: string;
  oldPath?: string;
  changeType: "modified" | "added" | "deleted" | "renamed";
  hunks: DiffHunk[];
}

export interface DiffHunk {
  header: string;
  lines: DiffLine[];
}

export interface DiffLine {
  type: "context" | "added" | "removed";
  oldLineNum?: number;
  newLineNum?: number;
  content: string;
}

export interface Comment {
  id: number;
  repoId: number;
  filePath: string;
  lineNum: number;
  body: string;
  isOutdated: boolean;
}

export type DiffLayout = "unified" | "split";
