---
id: T-0068
title: "Eval report: glue-aware per-function attribution, worst-cell parity tier and per-suite workbook closeness"
milestone: M0
lane: harness
status: done
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
- [x] `sole_call(called: &BTreeSet<String>) -> Option<&String>` lives next to `GLUE` and
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
- [x] Each `functions[]` entry has two new `Closeness` fields:
  - `excel`: every non-deviation Excel-evidence cell (`from_excel`) that calls the function;
  - `attributed`: the subset whose `sole_call` is the function.
  A unit test shows that a non-Excel cell (corpus-bench style) counts in neither.
- [x] Each `functions[]` entry has `tier` (`exact` | `ulp4` | `digits15` | `rate999` | `rate99` |
      `below99` | `unscored`) and `attribution` (`sole` | `all`). The tier is placed by
      `attributed`, or by `excel` when `attributed.scored() < EVIDENCE_FLOOR`. 99.9% reuses
      `RATE_NUM/RATE_DEN`, and 99% is one named constant. Unit tests cover exactly 99.9%, 19
      against 20 attributed cells, and a worst cell of `digits15`.
- [x] Each `suites[]` entry has `workbooks: {scored, all_exact, all_within15, any_visible}` over
      its cases with at least one scored cell, with a unit test.
- [x] report.md gains a "Parity tiers (glue-aware)" section: functions per tier, and how many use
      the fallback.
- [x] The Outcome gives the tier counts on a fresh `make eval` of `main` (poi, corpus-bench and
      calipers fetched) next to the assessment's (308 never visibly off, 127 below 99%), and
      explains any difference.
- [x] `make check` green, with no baseline or lock change; reviewers approve.

## Out of scope
- `run` metadata and the `v15_checkpoint` switch (T-0080).
- Changing `closeness`, `visible`, `blamed()`, the visible-parity lock or any gate.
- Rendering (T-0081).

## Outcome
- `closeness::sole_call` sits next to `GLUE` and `blamed()`, with the AC's table as a unit test
  (the `IF(AND(..))` row also asserts `blamed()` still gives AND and IF).
- New `crates/rustsheet-eval/src/attribution.rs` holds `Tier`, `Attribution`, `Workbooks`,
  `FunctionStat::add_attributed` / `place_tier` and the report.md section, so `RunReport::build`
  gains only the calls. `milestone_exit::RATE99_NUM` (990 over `RATE_DEN`) is the 99% constant.
- report.json: each `functions[]` entry gains `excel`, `attributed`, `tier` and `attribution`; each
  `suites[]` entry gains `workbooks`. A case's cells for `workbooks` are the ones the suite's
  `closeness` counts (in scope, deviations passing), so `any_visible` is 0 exactly when the suite
  has no visibly-off cell. No existing field, gate, baseline or lock changes.
- report.md's "Parity tiers (glue-aware)" section counts the in-scope catalog functions (those of
  the fifteen-digit checkpoint, 506). `functions[]` also holds 76 names the catalog does not know
  (user-defined names such as `MYFN`, `XYZ`); they get a tier but the section leaves them out.
- Tier counts over the 506 in-scope catalog functions, on a fresh full run (poi, corpus-bench and
  calipers fetched):

  | run | exact | ulp4 | digits15 | never visibly off | rate999 | ≥ 99.9% | rate99 | below99 | fallback |
  |---|---|---|---|---|---|---|---|---|---|
  | `main` 22588cf + this change | 300 | 25 | 5 | **330** | 7 | 337 | 71 | **98** | 3 |
  | `main` 2c52346 (the assessment's commit) + this change | 274 | 24 | 3 | **301** | 5 | 306 | 71 | **129** | 3 |
  | 2026-10-25 assessment | | | | 308 | | ~310 | | 127 | |

  - Most of the gap is the 94 commits since the assessment's 2c52346. Between the two runs, 51
    functions moved to a better tier, 33 of them out of `below99` (CORREL, PEARSON, RSQ, SLOPE,
    QUARTILE.INC, PRICE, MDURATION, TAN, IMREAL and others). 8 moved to a worse tier because new
    probes added cells (probes went from 1,425 to 1,530 cases and probes-numeric from 189 to
    263): T.DIST, VLOOKUP, GAMMALN, ERROR.TYPE, VALUE, XLOOKUP, IMABS, BASE.
  - At 2c52346 this code gives 301 never visibly off and 129 below 99%, against the assessment's
    308 and 127. Calipers matches exactly: 149 of its 171 scored workbooks are within 15 digits
    there (87.1%), the assessment's "87%". The rest of the gap is in how the assessment counted.
    Its one-off script is not in the repo; this report uses the `GLUE` list, `functions_in`,
    excludes deviations and keeps Excel-evidence cells only.
  - Rechecked on today's run: counting non-Excel cells too, or a control-flow-only glue list,
    moves "never visibly off" by at most 3.
- `workbooks` on today's run:
  - calipers 171 scored: 137 all exact, 153 within 15 digits (89.5%), 18 with a visible cell;
  - poi 115: 112 / 112 / 3;
  - corpus-bench 320: 312 / 313 / 7;
  - probes 1,524: 1,355 / 1,381 / 143.
- Gates:
  - cargo fmt, clippy (`--workspace --all-targets -D warnings`) and `cargo test --workspace`
    (5,295 passed, xlsx included) pass;
  - so do `make hygiene` (duplication 2951 = baseline), `make typos`, `make py-check` and
    `evidence-floor`;
  - `rustsheet-eval check` on every gated suite (poi and corpus-bench fetched) shows 0
    regressions, visible parity 4520 against the lock's 6039 (unchanged).
- Eval delta: none (report-only change).
