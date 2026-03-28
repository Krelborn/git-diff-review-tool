import type { DiffFile, DiffHunk, DiffLine } from "../types/diff";

// Matches: @@ -oldStart[,oldCount] +newStart[,newCount] @@ [context]
const HUNK_HEADER_RE = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

/**
 * Parse a raw unified diff string (as returned by `git diff`) into structured
 * DiffFile objects suitable for rendering.
 */
export function parseDiff(rawDiff: string): DiffFile[] {
  if (!rawDiff.trim()) return [];

  // Each file section starts with "diff --git". Split on that boundary,
  // keeping everything after the marker by re-splitting on the newline that
  // precedes the next "diff --git" line.
  const sections = rawDiff.split(/^(?=diff --git )/m).filter(Boolean);

  return sections.map(parseFileSection).filter((f): f is DiffFile => f !== null);
}

function parseFileSection(section: string): DiffFile | null {
  const lines = section.split("\n");

  let oldFilePath: string | null = null;
  let newFilePath: string | null = null;
  let renameFrom: string | undefined;
  let isRename = false;

  // Walk the header lines (before the first @@) to collect metadata.
  for (const line of lines) {
    if (line.startsWith("@@")) break;

    if (line.startsWith("--- ")) {
      const p = line.slice(4);
      oldFilePath = p === "/dev/null" ? null : stripPrefix(p, "a/");
    } else if (line.startsWith("+++ ")) {
      const p = line.slice(4);
      newFilePath = p === "/dev/null" ? null : stripPrefix(p, "b/");
    } else if (line.startsWith("rename from ")) {
      renameFrom = line.slice("rename from ".length);
      isRename = true;
    }
    // "rename to" is redundant with +++ b/path, so we skip it.
  }

  // Determine the canonical path and change type.
  let changeType: DiffFile["changeType"];
  if (oldFilePath === null && newFilePath !== null) {
    changeType = "added";
  } else if (oldFilePath !== null && newFilePath === null) {
    changeType = "deleted";
  } else if (isRename) {
    changeType = "renamed";
  } else {
    changeType = "modified";
  }

  // For deleted files the new path is /dev/null; use the old path as the
  // display path. For everything else, use the new path.
  const path = newFilePath ?? oldFilePath;
  if (path === null) return null; // no recognisable paths — skip

  const result: DiffFile = {
    path,
    changeType,
    hunks: parseHunks(lines),
  };

  if (renameFrom !== undefined) {
    result.oldPath = renameFrom;
  }

  return result;
}

function parseHunks(fileLines: string[]): DiffHunk[] {
  const hunks: DiffHunk[] = [];
  let hunkStart = -1;

  for (let i = 0; i < fileLines.length; i++) {
    if (fileLines[i].startsWith("@@")) {
      if (hunkStart !== -1) {
        const hunk = buildHunk(fileLines.slice(hunkStart, i));
        if (hunk) hunks.push(hunk);
      }
      hunkStart = i;
    }
  }

  if (hunkStart !== -1) {
    const hunk = buildHunk(fileLines.slice(hunkStart));
    if (hunk) hunks.push(hunk);
  }

  return hunks;
}

function buildHunk(lines: string[]): DiffHunk | null {
  if (lines.length === 0) return null;

  const header = lines[0];
  const match = header.match(HUNK_HEADER_RE);
  if (!match) return null;

  let oldNum = parseInt(match[1], 10);
  let newNum = parseInt(match[2], 10);

  const parsedLines: DiffLine[] = [];

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];

    // "\ No newline at end of file" — informational, skip.
    if (line.startsWith("\\ ")) continue;

    if (line.startsWith("+")) {
      parsedLines.push({ type: "added", newLineNum: newNum++, content: line.slice(1) });
    } else if (line.startsWith("-")) {
      parsedLines.push({ type: "removed", oldLineNum: oldNum++, content: line.slice(1) });
    } else if (line.startsWith(" ")) {
      parsedLines.push({
        type: "context",
        oldLineNum: oldNum++,
        newLineNum: newNum++,
        content: line.slice(1),
      });
    }
    // Blank lines or unrecognised prefixes between hunks are skipped.
  }

  return { header, lines: parsedLines };
}

function stripPrefix(s: string, prefix: string): string {
  return s.startsWith(prefix) ? s.slice(prefix.length) : s;
}
