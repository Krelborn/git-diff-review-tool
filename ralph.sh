#!/bin/bash
set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <iterations>"
  exit 1
fi

for ((i=1; i<=$1; i++)); do
  result=$(claude --dangerously-skip-permissions "@docs/requirements @docs/technical-design @docs/mockups/main-ui.html @progress.md \
1. Find the highest priority task to work on and work only on that task. \
This should be the one YOU decide has the highest priority, not necessarily the one that is first in the list. \
2. Check that the types check and the tests pass. \
3. Update the requirements document and github issue with the work that was done. \
4. Update the progress document with the work that was done and mark the github issue as complete. \
Use this to leave a note for the next person who works on this project. \
5. Make a git commit of the work that was done. \
ONLY WORK ON A SINGLE TASK. \
If, while implementing the task you notice that all tasks are complete, output <promise>COMPLETE</promise> and stop working. \
")

  echo "$result"

  if [[ "$result" == *"<promise>COMPLETE</promise>"* ]]; then
    echo "PRD complete after $i iterations."
    exit 0
  fi
done