---
id: T-0081
title: "Progress report parity section, all-functions page and docs/parity.md pointer"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0068, T-0070]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
docs/progress.md shows parity per function with the glue-aware tiers: functions per tier, tiers
by category, attributed cells by closeness, and the suites. docs/progress/functions.md lists
every function. docs/parity.md becomes a pointer, so the old parity page is retired and links
to it still work.

## Context
- SPEC-0022, the `parity` row of "Page layout", and "docs/parity.md".
- Inputs: report.json's T-0068 fields (`tier`, `attribution`, `attributed`, `excel`,
  `workbooks`) and T-0080's `v15_checkpoint`.
- This folds in T-0066 (dropped). The old generator recomputed the Phase 1 share
  (`within15_share`, `PHASE1_SHARE`); this task reads `v15_checkpoint` instead.
- Module: `progress_report/parity.py` in the package from T-0070.

## Acceptance criteria
- [ ] The `parity` block shows:
  - functions per `tier`;
  - tiers by category, as a table and a stacked SVG;
  - attributed cells by closeness;
  - the suites table, with cases, cells, bit-exact, ≤ 4 ulp, visibly off, `workbooks`,
    timeouts, and known failures (non-comment lines of `eval/baseline/<suite>.txt`).
  Python derives no verdict, bucket, tier or share threshold. A report without the T-0068 fields
  or `v15_checkpoint` fails loudly.
- [ ] Phase 1 numbers come from `v15_checkpoint` (`at_rate`, `in_scope`, per-function rate and
      `meets_rate`), with the strict count labelled as information. `within15_share` and
      `PHASE1_SHARE` are gone (T-0066's criterion).
- [ ] docs/progress/functions.md is written: every in-scope function with category, milestone,
      tier, attribution, attributed and Excel cells, and its within-15-digit share.
- [ ] docs/parity.md and docs/parity/functions.md become short static pointers to
      docs/progress.md. The three docs/parity/*.svg files are deleted. The generator writes
      neither file. docs/README.md links docs/progress.md.
- [ ] Tests on a synthetic report cover the tier table, the category table and a missing T-0068
      field.
- [ ] This PR commits docs/progress.md and docs/progress/ from a fresh `make eval` (poi,
      corpus-bench and calipers fetched). The Outcome compares the tier counts with the
      assessment's (308 never visibly off, 127 below 99%).
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- History and stale handling (T-0071).
- Goals and campaign (T-0072).
- Top-40 and evidence (T-0073).
