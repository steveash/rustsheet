---
id: T-0068
title: "Eval report: glue-aware per-function attribution, worst-cell parity tier and per-suite workbook closeness"
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
report.json carries two new things for the progress dashboard to render; it then computes no
verdicts itself:
- per in-scope function, the closeness of the Excel cells attributed to it under decision 0118,
  and its worst-cell parity tier on 15 digits;
- per suite, how many workbooks are all bit-exact, all within 15 digits, or have a visibly-off
  cell.

## Context
- SPEC-0022, "Glue-aware attribution lives in the Rust eval report", and decision 0118.
- Today `FunctionStat.closeness` counts a cell toward every function it calls, which would show
  485 of 506 functions as off. The 2026-10-25 assessment counted a cell only toward its sole
  non-routing call, falling back to all cells below 20 attributed cells. It found 308 functions
  never visibly off and 127 below 99%.
- T-2036 (done) put `Closeness` in the report. This task adds fields next to it. It changes no
  existing field and no verdict.
- The run metadata and the `v15_checkpoint` switch to these counts are T-0080.
- Code paths:
  - `RunReport::build` (crates/rustsheet-eval/src/report.rs, 966 lines; keep new logic out of
    it);
  - `closeness.rs`, which holds `GLUE` and `blamed()`;
  - `milestone_exit::RATE_NUM` / `RATE_DEN`;
  - `evidence_floor::EVIDENCE_FLOOR`.

## Acceptance criteria
- [ ] `sole_call(called: &BTreeSet<String>) -> Option<&String>` lives next to `GLUE` and
      `blamed()` in crates/rustsheet-eval/src/closeness.rs, or in a new `attribution.rs` beside
      it. It takes the same distinct-call set `blamed()` takes. It returns:
  - the one function of the set not in `GLUE`;
  - when every call is glue, the set's only element;
  - otherwise `None`.

  Unit tests:

  | Formula | Result |
  |---|---|
  | `IF(A1,GAMMA(B1))` | GAMMA |
  | `GAMMA(1)+GAMMA(2)` | GAMMA (repeated calls are one) |
  | `SUM(A1:A3)` | SUM |
  | `IF(A1,1,2)` | IF |
  | `GAMMA(BETA.DIST(..))` | `None` |
  | `IF(AND(A1,B1),1,2)` | `None` |

  The `IF(AND(..))` test also asserts that `blamed()` still returns IF and AND, the difference
  decision 0118 records.
- [ ] Each `functions[]` entry has two new `Closeness` fields:
  - `excel`: every non-deviation Excel-evidence cell (`from_excel`) that calls the function;
  - `attributed`: the subset whose `sole_call` is the function.
  A unit test shows that a non-Excel cell (corpus-bench style) counts in neither.
- [ ] Each `functions[]` entry has `tier` (`exact` | `ulp4` | `digits15` | `rate999` | `rate99` |
      `below99` | `unscored`) and `attribution` (`sole` | `all`). The tier is placed by
      `attributed`, or by `excel` when `attributed.scored() < EVIDENCE_FLOOR`. 99.9% reuses
      `RATE_NUM/RATE_DEN`, and 99% is one named constant. Unit tests cover exactly 99.9%, 19
      against 20 attributed cells, and a worst cell of `digits15`.
- [ ] Each `suites[]` entry has `workbooks: {scored, all_exact, all_within15, any_visible}` over
      its cases with at least one scored cell, with a unit test.
- [ ] report.md gains a "Parity tiers (glue-aware)" section: functions per tier, and how many use
      the fallback.
- [ ] The Outcome gives the tier counts on a fresh `make eval` of `main` (poi, corpus-bench and
      calipers fetched) next to the assessment's (308 never visibly off, 127 below 99%), and
      explains any difference.
- [ ] `make check` green, with no baseline or lock change; reviewers approve.

## Out of scope
- `run` metadata and the `v15_checkpoint` switch (T-0080).
- Changing `closeness`, `visible`, `blamed()`, the visible-parity lock or any gate.
- Rendering (T-0081).
