---
id: T-0084
title: "make new-task and make new-decision break on a TITLE containing a double quote"
milestone: M0
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`make new-task` and `make new-decision` accept any title text, double quotes and `;` included, and pass it to the
script unchanged.

## Context
Gap triage 2026-10-27: `make new-task MILESTONE=M5 LANE=functions-text TITLE='TEXT of a text value with an empty text
section (";;;") is empty text, not the text'` failed with `/bin/sh: 1: Syntax error: ";;" unexpected` and no task was
written. The recipes in the Makefile wrap the title as `--title "$(TITLE)"`, so a `"` in TITLE ends the shell string
early. Task and decision titles often quote Excel formulas or format codes (`"0;-0"`, `TEXT(x,"@")`), so the
workaround of calling `python3 scripts/tasks.py new` directly should not be needed. The same pattern can affect other
targets that splice free text (`CASE=` in `eval-show` is less likely to hold quotes; check them all).
One fix: pass the value through the environment (`--title "$$TITLE"`). GNU make exports command-line variables to
recipes. Another fix: escape it with `$(subst ...)`. Pick one and use it for every free-text variable.

## Acceptance criteria
- [ ] `make new-task MILESTONE=M0 LANE=harness TITLE='a "quoted" ;;; title' SIZE=S` writes a task whose front-matter title is
      `a "quoted" ;;; title` (test it in a scratch checkout or with a dry-run flag, and delete the file it writes).
- [ ] The same for `make new-decision TITLE='a "quoted" title'`.
- [ ] A test (scripts/ or a Makefile smoke test in `make check`) covers a title with `"`, `;`, `$` and a backtick.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `scripts/tasks.py new` itself beyond what the test needs.

## Outcome (draft, from the sonnet attempt; PR #2245)
`new-task` and `new-decision` now read `MILESTONE`, `LANE` and `TITLE` from the environment (`"$$TITLE"`), so `"`,
`;`, backticks and parentheses pass through unchanged. A single `$` on the make command line is still expanded by make
itself (write `$$`); the help text says so. `scripts/test_makefile_titles.py` covers it (run by `make hygiene`).
Eval delta: none. Other targets splice no free text except `CASE=` in `eval-show` (a path; left as is).

## Handoff (resumed)
- Tried: `new-task`/`new-decision` recipes now use `"$$TITLE"`, `"$$LANE"`, `"$$MILESTONE"`; added
  `scripts/test_makefile_titles.py` (3 tests, pass; `make check` green locally; security/architecture/quality reviewers APPROVE).
- Failed: CI `hygiene` job, `scripts/check_governance.py origin/main`: "protected paths changed without a decision
  record in docs/decisions/: Makefile, scripts/test_makefile_titles.py". `make check` did not catch it locally.
- Next: `make new-decision TITLE="Makefile passes free-text variables via the environment"` (record why, and that a
  single `$` must be written `$$`), commit it with the branch, confirm `python3 scripts/check_governance.py origin/main`
  passes, set `status: done`, rename the Outcome heading back, then merge per work-loop step 6.
