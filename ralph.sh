#!/bin/bash
set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <iterations>"
  exit 1
fi

# jq filter to extract streaming text from assistant messages
stream_text='select(.type == "assistant").message.content[]? | select(.type == "text").text // empty | gsub("\n"; "\r\n") | . + "\r\n\n"'

# jq filter to extract final result
final_result='select(.type == "result").result // empty'

for ((i=1; i<=$1; i++)); do
  tmpfile=$(mktemp)
  trap "rm -f $tmpfile" EXIT

  claude --dangerously-skip-permissions \
    --verbose \
    --print \
    --output-format stream-json "@docs/requirements @docs/technical-design @docs/mockups/main-ui.html @progress.md \
    1. Find the highest priority task to work on and work only on that task. \
    This should be the one YOU decide has the highest priority, not necessarily the one that is first in the list. \
    2. Check that the types check and the tests pass. \
    3. Update the requirements document and github issue with the work that was done. \
    4. Update the progress document with the work that was done and mark the github issue as complete. \
    Use this to leave a note for the next person who works on this project. \
    5. Make a git commit of the work that was done. \
    ONLY WORK ON A SINGLE TASK. \
    If, while implementing the task you notice that all tasks are complete, output <promise>COMPLETE</promise> and stop working. \
    " \
  | grep --line-buffered '^{' \
  | tee "$tmpfile" \
  | jq --unbuffered -rj "$stream_text"

  result=$(jq -r "$final_result" "$tmpfile")

  if [[ "$result" == *"<promise>COMPLETE</promise>"* ]]; then
    echo "Ralph complete after $i iterations."
    exit 0
  fi
done