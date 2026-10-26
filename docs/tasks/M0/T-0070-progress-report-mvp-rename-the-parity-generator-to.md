---
id: T-0070
title: "Progress report MVP: rename the parity generator to make progress-report, with docs inputs, the status block and judgment-block preservation"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0069, T-0080]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
`make progress-report` exists and writes docs/progress.md with these blocks:
- the `status` block;
- the `bottom-line`, `readiness` and `risks` judgment blocks, preserved across runs;
- the `method` block.

It is the existing parity generator, renamed into a package, not copied. The docs it reads are
classified as code for CI.

## Context
- SPEC-0022, sections "Page layout", "Judgment blocks" and "Inputs and generator".
- The current generator is tools/src/rustsheet_tools/parity_report.py with parity_svg.py, tests
  in tools/tests/test_parity_report.py, and `make parity-report` (decision 0079).
- **Docs reads.** The generator reads docs/milestones.md, docs/progress.md and
  docs/progress/history.json. Decision 0079 says "nothing under docs/ is read by code", and
  decisions 0049 and 0066 skip the rust, eval and python CI jobs for docs-only PRs.
  `scripts/check_docs_readers.py` fails on a docs literal in tools/ unless its path is in
  `CODE_READ_PATHS`.

  Chosen approach (SPEC-0022): add `docs/milestones.md`, `docs/progress.md` and
  `docs/progress/` to `CODE_READ_PATHS` in scripts/ci_changes.py. Steward PRs that change the
  page or its history then still run the python job.
- Protected paths: the Makefile and scripts/ci_changes.py. Both are covered by one decision
  record from `make new-decision`.
- The parity section, functions.md and the parity.md pointer are T-0081.

## Acceptance criteria
- [x] The rename is done with `git mv`, and no copy of the old module is left:
  - parity_report.py becomes the package `tools/src/rustsheet_tools/progress_report/`, with
    `__init__` (CLI), `model` (input loading and validation) and `blocks` (markers);
  - parity_svg.py becomes `progress_report/svg.py`;
  - the entry point becomes `rustsheet-progress-report`;
  - the tests become `test_progress_report.py`.
- [x] `make progress-report` replaces `make parity-report`. It writes
      `eval/runs/latest/tasks.json` with `scripts/tasks.py stats --json` (T-0069), then runs the
      generator with `--out-dir ../docs`.
- [x] scripts/ci_changes.py `CODE_READ_PATHS` gains `docs/milestones.md`, `docs/progress.md` and
      `docs/progress/`. scripts/test_ci_changes.py, or the existing classifier test, shows that a
      PR changing only docs/progress/history.json is `code=true`. `make hygiene`
      (check_docs_readers) passes with the generator's literals.
- [x] One decision record covers the Makefile and ci_changes.py. It amends decision 0079:
  - the generator now reads the three docs paths, which are classified as code;
  - docs/parity.md is replaced (T-0081);
  - decision 0079's "no CI freshness gate on the page" still holds.
- [x] The `status` block shows:
  - run date and commit from report.json `run` (T-0080);
  - suites run and skipped;
  - links to docs/status.md and docs/progress/functions.md.
- [x] The judgment blocks `bottom-line`, `readiness` and `risks` follow SPEC-0022. Tests cover:
  - text kept verbatim;
  - a placeholder for a missing block;
  - failure on an unknown name, on nested or unbalanced markers, and on a marker inside judgment
    text;
  - the "not yet updated for this run" line when the as-of does not name the run.
- [x] Two runs on the same inputs write byte-identical files (test).
- [ ] `make py-check`, `make hygiene` and `make typos` green; `make check` green; reviewers
      approve, with explicit architecture and security approval of the protected paths.

## Out of scope
- The parity section, functions.md and the docs/parity.md pointer (T-0081).
- History, deltas, trends and stale handling (T-0071).
- Goals and campaign sections (T-0072).
- Top-40 and evidence (T-0073).
- `--check` (T-0074).
- Writing judgment text (T-0075).

## Outcome
- `git mv` moved parity_report.py to `progress_report/parity.py`, parity_svg.py to
  `progress_report/svg.py` and test_parity_report.py to test_progress_report.py. No copy is left.
  The split:
  - `model.py`: input loading and validation, `compute`, and the checked `run` record;
  - `blocks.py`: markers and judgment preservation;
  - `page.py`: layout and the `status` and `method` blocks;
  - `markdown.py`: shared Markdown helpers;
  - `__init__.py`: the CLI `rustsheet-progress-report`.
- `make progress-report` replaces `make parity-report`. It writes eval/runs/latest/tasks.json
  through a temporary file, then runs the generator with `--out-dir ../docs`. `COMMIT=` and
  `RUN_DATE=` are gone: the date and commit come from report.json `run`, and a malformed one is
  an error. Every input, the existing page included, is checked before any file is written.
- Decision 0120 amends 0079. `CODE_READ_PATHS` gains docs/milestones.md, docs/progress.md and
  docs/progress/. scripts/test_ci_changes.py shows that a PR touching only
  docs/progress/history.json is `code=true`.
- Judgment blocks are kept byte for byte, CRLF included: the page is read and written as bytes.
  These stop the run: an unknown block name of either kind, a duplicate name, nested or
  unbalanced markers, a malformed marker comment, an unterminated comment, and any marker inside
  judgment text. A missing block gets a placeholder. A stale as-of (other date or commit, or none)
  adds one line under the block's heading. tools/tests/test_progress_page.py covers each case and
  byte-identical reruns.
- Gates: `make py-check`, `make hygiene` and `make typos` pass, and so do the
  rustsheet-eval tests. The only Rust change is the path in closeness.rs's module comment.
- For T-0081 and later:
  - until T-0081 replaces it, the target still writes docs/parity.md and docs/parity/;
  - nothing reads tasks.json yet (T-0072);
  - docs/progress.md is not committed yet: the first committed page is T-0081's.
