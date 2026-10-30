# 0156 — Makefile passes free-text variables via the environment

- **Status:** accepted (2026-10-29, T-0084).
- **Context:** `make new-task ... TITLE='... (";;;") ...'` failed with `/bin/sh: Syntax error: ";;"
  unexpected` and wrote no task: the recipes spliced `$(TITLE)` into a double-quoted shell string,
  so a `"` in the title ended it early and the rest ran as shell code. Task and decision titles
  often quote Excel formulas and format codes, so they hold `"`, `;`, `(` and backticks.
- **Decision:** a recipe reads a free-text variable (`TITLE`, `LANE`, `MILESTONE` in `new-task` and
  `new-decision`) from the environment as `"$$NAME"`, never as `"$(NAME)"`. GNU make exports
  command-line variables to recipes, so the shell sees the value as one word with no re-parsing.
  A literal `$` on the make command line is still expanded by make itself before export and must
  be written `$$`; the `new-task` help text says so. `scripts/test_makefile_titles.py` (run by
  `make hygiene`) pins both recipes.
- **Consequences:** titles with quotes and shell metacharacters work through `make` without calling
  `scripts/tasks.py new` directly. A new Makefile target that takes free text follows the same
  rule. `CASE=` in `eval-show` is a path and is left as is.
