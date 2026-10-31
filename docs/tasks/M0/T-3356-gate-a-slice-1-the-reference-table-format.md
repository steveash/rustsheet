---
id: T-3356
title: "Gate A slice 1: the reference-table format, its reader and the make check gate (from the Gate A spec)"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3355]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
The Gate A reference-table format of the Gate A spec (T-3355) exists, with a reader and the check `make check` runs:
a point fails when it is neither within ε = 1e-9 relative of its math value (absolute 1e-300 near 0) nor within 15
significant digits of Excel's value, or when one side is an error and the other a number (decision 0188 R2).

## Context
- Decision 0188 R1(a) and R2; the format, location and runner are the spec's (T-3355).
- Reuse the eval engine adapter and `Visible::is_off` (crates/rustsheet-eval), never a second 15-digit rule.
- SPEC-0029 decides the design: the `eval/accuracy/` manifest resolved by `Suite::load_named` (for `cases` and
  `show`, never `make eval`), batches `tables/<FUNCTION>/<batch>.case.json` with oracle and `rustsheet-math/v1` math
  sidecars, `envelope` (the one definition T-3359 and T-3380 reuse), `known-failures.tsv` (stale lines fail),
  `math-over-excel.tsv` (anchors checked by `deviations::check_entry` given the status, Excel off by > 1e-4 on a filled
  marked point), the summary and `--json eval/runs/accuracy.json`, a 30 s budget, and the `make eval-gates` wiring
  (Makefile is protected: a decision record).

## Acceptance criteria
- [x] The reader and the check exist as the spec says, with unit tests for a passing point, a point within ε of the
      math only, within 15 digits of Excel only, outside both, an error against a number, and an R3 point checked
      against the math only.
- [x] `make check` runs the check and fails on a failing point (shown by a test fixture, not a real table).
- [x] The check's runtime on the slice's fixture tables is reported in the Outcome against the spec's budget.
- [x] docs/architecture/eval-harness.md describes the check and links SPEC-0029.
- [x] `make check` green; reviewers approve.

## Out of scope
- The generator and the oracle fill (T-3357); the real tables (T-3358).

## Outcome
- **What changed.** `rustsheet-eval accuracy` (crates/rustsheet-eval/src/accuracy.rs with
  `accuracy/{envelope,math,lists}.rs`): it loads `eval/accuracy/` through the new
  `Suite::load_named` (which `cases` and `show` now use too, and which refuses a suite named
  `accuracy` under eval/suites), runs each batch once through the new
  `runner::run_case_values` (shares `run_case`'s preparation, panic isolation and timeout; also
  returns the engine value of every check cell, pending ones included), classifies each point with
  `accuracy::envelope::envelope` (Excel side = `!Visible::is_off` on the runner's own scored cell),
  applies `known-failures.tsv` (stale lines fail) and `math-over-excel.tsv` (anchors through
  `deviations::check_entry_status`, the existing anchor check now parameterized by status;
  G = 1e-4 evidence on filled points), prints the summary and failures worst first, and writes
  `--json` (default `eval/runs/accuracy.json`). `make eval-gates` runs it after
  `check --evidence-floor` (decision 0193). `eval/accuracy/` holds the manifest and two empty
  lists, so the check passes with 0 points until T-3358.
- **Tests.** Unit tests of `envelope` for every case of the spec's test plan (both, ε only,
  15 digits only, outside, absolute floor, error vs number either way, two error codes, domain
  error matched, R3 point within 15 digits of Excel but not ε, missing/stale math, domain error vs
  Excel number), of the math sidecar reader, the lists, `load_named` and the known-failure logic.
  End to end (`tests/accuracy_gate.rs`) on the fixture repository
  `crates/rustsheet-eval/tests/data/accuracy-fixture` (made-up values, not a real table): one known
  failure, one stale known-failure line, one failing point and one stale math value give exit 1
  with the expected lines; without them exit 0; an R3 mark without evidence fails.
- **Runtime against the 30 s budget.** Fixture: 0.00 s (9 points, release build). A throwaway
  synthetic set of 50,400 points (420 tables × 120 one-call formulas, filled, with math; not
  committed) took 0.31 s for the check as printed and 0.8 s wall including writing the 16 MB JSON,
  on a 4-core container.
- **Choices where the spec is silent.**
  - Point id: `runner::cell_id` of the case id *less* `.case.json` (`accuracy/T.INV/s1!Calc!A1`),
    as the spec's examples show; `show` still takes the full case id.
  - Points are a batch's `check` cells (every expected key when `check` is empty, the runner's
    rule). A batch with no check cells, an unreadable case, a bad path (not
    `tables/<FUNCTION>/<batch>.case.json`) or an unreadable math sidecar is a table defect.
  - The math sidecar path is the oracle sidecar's with `.math.json`, so `oracle_dir` applies.
    `f` is compared without a leading `=` (either form accepted). `source.input_sha256` is read
    but not verified (immutability is T-3379's).
  - While pending, an engine error against a math number, or a number against a math domain
    error, is `ErrorMismatch` (the math stands in for Excel as the reference).
  - Excel's side counts only for an oracle value from Excel (`from_excel`); a hand value is pending.
  - `math-over-excel.tsv` cites entries as `docs/excel-semantics/<file>.md#<anchor>` (the
    `deviations` form). Lines naming non-accuracy eval cells get only the anchor check here; a mark
    on an accuracy id no table holds fails, and a filled mark needs two numbers with Excel off by > G.
  - Known-failure lines need a `T-NNNN` owner and a non-empty note; a line whose point no table
    holds is stale. A listed point that is a table defect still fails as a defect.
  - A missing list file is empty (deleting one only makes the gate stricter).
  - The summary prints `missing tables n/a (T-3379)` (the count and `exempt.tsv` are T-3379's);
    its seconds cover the check, not the JSON write.
- **Review fixes.** An unsupported engine result (`#NAME?`, `#N/IMPL`, verdict `Unsupported`)
  never counts as inside on the Excel side; an R3 point drops the Excel side only where Excel gives a
  number, so Excel's errors are still matched (decision 0188 R3); the R3 evidence test has no
  absolute floor (nonzero Excel against an exact-0 math counts as more than G). The plain eval path
  builds no check cells (`prepare` takes the `values` flag), and `--json` streams a borrowed wrapper
  through a `BufWriter`. String-keyed math lookups and the timeout default are left for T-3358 to
  measure.
- **Follow-ups.** None new: ci_changes routing, the missing-table count, protected paths and the
  known-failure line rules are T-3379; the generator T-3357; real tables T-3358.
