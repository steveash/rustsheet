---
id: T-1784
title: "Move lift_with_cost into functions/lift/driver.rs with no result change (SPEC-0011 step 1a)"
milestone: M9
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: [T-1767]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
`lift_with_cost`, `CriteriaArg`, `count_walks` and `is_lifted` live in `crates/rustsheet-engine/src/functions/lift/driver.rs` as SPEC-0011's `lift_values` and `LiftedArg`, with no result change. This is the pure-move first half of the driver work; T-1770 adds the table.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "The driver" and "Today's lifting code".
- Callers today: the *IF(S) functions (`lift_criteria` in statistical/if_ifs.rs, which stays as the front end), AGGREGATE (aggregate.rs), IFS and SWITCH (logical/mod.rs, switch.rs, `is_lifted`), `lookup_each` and XLOOKUP.
- Changes in this task:
  - The `previous`-key memo becomes `LiftOptions::reuse_equal_runs`, comparing grid slots at the previous position through `bcast_get` instead of building a key `Vec` per element.
  - The cost argument becomes `LiftCost` (`Walk` and `Cells` for the existing callers).
  - The kernel becomes `&mut dyn FnMut`.
  - The argument list stays today's `Vec`; T-1770 introduces `ArgBuf`.
- No new behavior: the same shapes, bounds and results.

## Acceptance criteria
- [x] `make eval`: 0 fixes and 0 regressions in every suite.
- [x] `rg -n "fn lift_with_cost|enum CriteriaArg|fn count_walks|fn is_lifted" crates/rustsheet-engine/src` finds nothing outside functions/lift/.
- [x] A criterion bench in crates/rustsheet/benches/ covers a lifted whole-column COUNTIFS (`=SUM(COUNTIFS(A:A,A1:A100000))` over a filled column, the `reuse_equal_runs` path) and a SUMIFS with an array criterion. Both run on main and on the branch in release, are within 5%, and the numbers are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The argument-kind table, `lift_call` and any new consumer (T-1770).

## Outcome
`lift_with_cost`, `CriteriaArg`, `count_walks` and `is_lifted` are now `Model::lift_values`, `LiftedArg` (with `LiftedArg::is_grid`, also used by `lookup_each` and XLOOKUP in place of their own `matches!`), `LiftCost` (`Cells`, `Walk`) and `LiftOptions::reuse_equal_runs` in `functions/lift/{mod.rs,driver.rs}`. The kernel is `&mut dyn FnMut`, and the arguments stay a `Vec<CalcResult>`: the Range/Array → `LiftedArg` read is inside `lift_values` until T-1770's read phase and `ArgBuf`. `reuse_equal_runs` compares the grid slots at the previous position through `bcast_get` (no per-element key `Vec`), and one value buffer is kept for the call (scalar slots are written once). The cost stays today's: per-element cost × lifted arguments × computed elements. SPEC-0011's "elements × cells" for `Cells`/`Walk` would change which calls are refused, so T-1770 owns that change if it wants it. All callers (*IF(S), AGGREGATE, IFS, SWITCH, `lookup_each`, XLOOKUP) pass `LiftOptions::REUSE_EQUAL_RUNS`, as before. Unit tests cover `count_walks` with reuse on and off. No eval delta: `make eval-check` showed 0 fixed and 0 regressions in every baselined suite. The full `make eval` report.json on the parent and on the branch is identical, cell by cell, in all suites, including the suites with no baseline (gen-m6 to gen-m9, probes-numeric). `functions/mod.rs` stays at its locked size: one comment was folded into one line to make room for `mod lift;`.

Bench `crates/rustsheet/benches/lift.rs` (`cargo bench -p rustsheet --bench lift`, release, 100,000 rows in 10 runs of 10,000 equal values, median of 5 recalcs). The two binaries were run interleaved 4 times each on a shared 4-core box; the figure is the median of the 4 medians:

| Case | parent (4e0abfb6) | branch | Δ |
|---|---|---|---|
| `=SUM(COUNTIFS(A:A,A1:A100000))` (`reuse_equal_runs`) | 1.264 s | 1.222 s | −3.3% |
| `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))` | 1.073 s | 1.111 s | +3.5% |

Both are within 5% and within the run-to-run noise (±8%). The criteria walks dominate the time. The bench is a `harness = false` timing like `aggregate.rs`, not criterion, because criterion is not a dependency.
