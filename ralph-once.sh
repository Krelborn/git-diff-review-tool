set -e

claude --dangerously-skip-permissions "@docs/requirements @docs/technical-design @docs/mockups/main-ui.html @progress.md \
1. Find the highest priority task to work on and work only on that task. \
This should be the one YOU decide has the highest priority, not necessarily the one that is first in the list. \
2. Check that the types check and the tests pass. \
3. Update the requirements document with the work that was done. \
4. Update the progress document with the work that was done. \
Use this to leave a note for the next person who works on this project. \
5. Make a git commit of the work that was done. \
ONLY WORK ON A SINGLE TASK. \
If, while implementing the task you notice that all tasks are complete, output <promise>COMPLETED</promise> and stop working. \