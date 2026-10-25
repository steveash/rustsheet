---
id: T-0070
title: "Progress report MVP: rename the parity generator to make progress-report, write docs/progress.md with attributed tiers and judgment blocks, turn docs/parity.md into a pointer"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0068, T-0069]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
`make progress-report` writes docs/progress.md and docs/progress/ from the eval report. The page
has:
- the `status` block;
- the judgment blocks, preserved across runs;
- the glue-aware parity section, with tiers, categories, cells and suites;
- the method notes;
- an all-functions table.
docs/parity.md becomes a pointer to it. The old generator is renamed and extended, not copied.

## Context
- SPEC-0022, sections "Page layout", "Judgment blocks", "Inputs and generator" and
  "docs/parity.md".
- The current generator is tools/src/rustsheet_tools/parity_report.py with parity_svg.py, tests
  in tools/tests/test_parity_report.py, and `make parity-report` (decision 0079).
- It still recomputes the Phase 1 share (`within15_share`, `PHASE1_SHARE`). This task folds in
  T-0066 (dropped) and reads `v15_checkpoint` instead.
- The Makefile is protected, so the target change needs a decision record that amends decision
  0079. No freshness gate is added.

## Acceptance criteria
- [ ] The package and its tests move with `git mv`:
  - parity_report.py becomes the package `tools/src/rustsheet_tools/progress_report/`, laid out
    as SPEC-0022 says (`__init__`, `model`, `blocks`, `parity`; the other modules come with
    their tasks);
  - parity_svg.py becomes `progress_report/svg.py`;
  - the entry point becomes `rustsheet-progress-report`;
  - the tests become `test_progress_report.py`.
  No copy of the old module is left.
- [ ] `make progress-report` replaces `make parity-report`. It writes
      `eval/runs/latest/tasks.json` with `scripts/tasks.py stats --json` (T-0069), then runs the
      generator. A new decision record amends decision 0079.
- [ ] The parity section renders the report's T-0068 fields: functions per `tier`, tiers by
      category (table and stacked SVG), attributed cells by closeness, and the suites table with
      workbooks, timeouts and known failures (non-comment lines of `eval/baseline/<suite>.txt`).
      Python derives no verdict, bucket, tier or share threshold. A report without the T-0068
      fields or `v15_checkpoint` fails loudly.
- [ ] The Phase 1 numbers come from `v15_checkpoint` (`at_rate`, `in_scope`, per-function
      `cells`, `visible`, `meets_rate`). `within15_share` and `PHASE1_SHARE` are gone (T-0066's
      criterion).
- [ ] The judgment blocks `bottom-line`, `readiness` and `risks` follow SPEC-0022. Tests cover:
  - text kept verbatim;
  - a placeholder when a block is missing;
  - failure on an unknown name and on nested or unbalanced markers;
  - failure on a marker inside judgment text;
  - the "not yet updated for this run" line when the as-of does not name the run.
- [ ] docs/progress/functions.md is written: every in-scope function with category, milestone,
      tier, attribution, attributed and Excel cells, and its within-15-digit share.
- [ ] docs/parity.md and docs/parity/functions.md become short static pointers to
      docs/progress.md, and the three docs/parity/*.svg files are deleted. docs/README.md links
      docs/progress.md.
- [ ] Two runs on the same inputs write byte-identical files (test).
- [ ] This PR commits docs/progress.md and docs/progress/ from a fresh `make eval` (poi,
      corpus-bench and calipers fetched). Its Outcome compares the tier counts with the
      assessment's.
- [ ] `make py-check`, `make hygiene` and `make typos` green; `make check` green; reviewers approve.

## Out of scope
- History, deltas, trends and stale handling (T-0071).
- Goals and campaign sections (T-0072).
- Top-40 and evidence (T-0073).
- `--check` (T-0074).
- Writing judgment text (T-0075).
