---
id: T-0068
title: "Eval report: glue-aware per-function attribution, worst-cell parity tier, per-suite workbook closeness and skipped suites"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0067]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
report.json carries, per in-scope function, the closeness of the Excel cells where it is the
formula's only non-glue call, and its worst-cell parity tier on 15 digits. Per suite it carries
how many workbooks are all bit-exact, all within 15 digits, or have a visibly-off cell. It also
records which commit the run was on and which suites were skipped. The progress dashboard renders
these and computes no verdicts.

## Context
- SPEC-0022, "Glue-aware attribution lives in the Rust eval report".
- Today `FunctionStat.closeness` counts a cell toward every function it calls. That would show
  485 of 506 functions as off. The 2026-10-25 assessment instead counted a cell only toward its
  sole non-routing call (`closeness::GLUE`), with a fallback to all cells below 20 attributed
  cells: 308 functions never visibly off, 127 below 99%.
- T-2036 (done) put `Closeness` in the report. This task adds fields next to it. It changes no
  existing field and no verdict.
- Code paths:
  - `RunReport::build` (crates/rustsheet-eval/src/report.rs, 966 lines, so put the new logic in a
    new module, for example `attribution.rs`);
  - `formula_calls.rs`;
  - `closeness.rs`;
  - `milestone_exit::RATE_NUM` / `RATE_DEN`;
  - `evidence_floor::EVIDENCE_FLOOR`;
  - the "not fetched" skip in main.rs.

## Acceptance criteria
- [ ] `formula_calls::sole_call` returns the one non-`GLUE` function of a formula's calls, or the
      single function of a glue-only formula, and `None` otherwise. Unit tests cover:
      `IF(A1,GAMMA(B1))` → GAMMA; `SUM(A1:A3)` → SUM; `IF(A1,1,2)` → IF;
      `GAMMA(BETA.DIST(..))` → `None`; `IF(SUM(A1),1)` → `None`.
- [ ] Each `functions[]` entry has two new `Closeness` fields:
  - `excel`: every non-deviation Excel-evidence cell (`from_excel`) that calls the function;
  - `attributed`: the subset whose `sole_call` is the function.
  A unit test shows that a corpus-bench-style non-Excel cell counts in neither.
- [ ] Each `functions[]` entry has `tier` (`exact` | `ulp4` | `digits15` | `rate999` | `rate99` |
      `below99` | `unscored`) and `attribution` (`sole` | `all`). It is placed by `attributed`,
      or by `excel` when `attributed.scored() < EVIDENCE_FLOOR`. 99.9% reuses `RATE_NUM/RATE_DEN`
      and 99% is one named constant. Unit tests cover exactly 99.9%, 19 against 20 attributed
      cells, and a function whose worst cell is `digits15`.
- [ ] Each `suites[]` entry has `workbooks: {scored, all_exact, all_within15, any_visible}` over
      its cases with at least one scored cell, with a unit test.
- [ ] report.json has `run: {commit, date, skipped_suites: [{name, reason}]}`:
  - `commit` is `git rev-parse --short=8 HEAD`, or null when git fails, and never an error;
  - `date` is the UTC run date;
  - `skipped_suites` lists each suite under eval/suites that did not run, with its reason.
- [ ] report.md gains a "Parity tiers (glue-aware)" section: functions per tier and how many use
      the fallback.
- [ ] The Outcome gives the tier counts on a fresh `make eval` of `main` with poi, corpus-bench
      and calipers fetched, next to the assessment's (308 never visibly off, 127 below 99%), and
      explains any difference.
- [ ] `make check` green, with no baseline or lock change; reviewers approve.

## Out of scope
- Changing `closeness`, `visible`, `v15_checkpoint`, the visible-parity lock or any gate.
- Changing the Phase 1 measure to the glue-aware one: that is an owner decision (SPEC-0022, "Open
  question").
- Rendering the fields on a page (T-0070).
