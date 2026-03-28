import { type JSX } from "react";

import { useAppStore } from "../store";
import { type DiffLine } from "../types/diff";

function lineIndicator(type: DiffLine["type"]): string {
  if (type === "added") {
    return "+";
  }
  if (type === "removed") {
    return "−";
  }
  return "";
}

function gutterClass(type: DiffLine["type"]): string {
  if (type === "added") {
    return "line-num added-gutter";
  }
  if (type === "removed") {
    return "line-num removed-gutter";
  }
  return "line-num";
}

export function UnifiedDiffView(): JSX.Element {
  const fileDiffs = useAppStore((s) => s.fileDiffs);
  const selectedFilePath = useAppStore((s) => s.selectedFilePath);

  if (selectedFilePath === null) {
    return (
      <div
        style={{
          alignItems: "center",
          color: "var(--text-muted)",
          display: "flex",
          flex: 1,
          justifyContent: "center",
        }}
      >
        Select a file to review
      </div>
    );
  }

  const diffFile = fileDiffs.find((f) => f.path === selectedFilePath);

  if (diffFile === undefined) {
    return (
      <div
        style={{
          alignItems: "center",
          color: "var(--text-muted)",
          display: "flex",
          flex: 1,
          justifyContent: "center",
        }}
      >
        No diff available for this file
      </div>
    );
  }

  return (
    <div className="diff-viewer">
      {diffFile.hunks.map((hunk, hunkIndex) => (
        <div key={hunkIndex}>
          <div className="hunk-header">{hunk.header}</div>
          {hunk.lines.map((line, lineIndex) => (
            <div className={`diff-line ${line.type}`} key={lineIndex}>
              <div className={gutterClass(line.type)}>
                {line.oldLineNum !== undefined ? line.oldLineNum : ""}
              </div>
              <div className={gutterClass(line.type)}>
                {line.newLineNum !== undefined ? line.newLineNum : ""}
              </div>
              <div className="line-indicator">{lineIndicator(line.type)}</div>
              <div className="line-content">{line.content}</div>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
