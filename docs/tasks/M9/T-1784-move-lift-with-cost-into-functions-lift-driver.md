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
- [x] A `harness = false` bench in crates/rustsheet/benches/ (a min/median/spread timer like benches/aggregate.rs, sharing benches/common/; criterion is not a dependency) covers a lifted whole-column COUNTIFS (`=SUM(COUNTIFS(A:A,A1:A100000))` over a filled column, the `reuse_equal_runs` path) and a SUMIFS with an array criterion, plus two SWITCH cases where the driver dominates (`=SUM(SWITCH(A1:A1000000,0,1,1,2,3))` and the text-returning `=ROWS(SWITCH(A1:A1000000,0,"a","b"))`). All run on main and on the branch in release, are within 5% with noise below 5%, and the numbers are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The argument-kind table, `lift_call` and any new consumer (T-1770).

## Outcome
`lift_with_cost`, `CriteriaArg`, `count_walks` and `is_lifted` are now `Model::lift_values`, `LiftedArg` (with `LiftedArg::is_grid`, also used by `lookup_each` and XLOOKUP in place of their own `matches!`), `LiftCost` (`Cells`, `Walk`) and `LiftOptions::reuse_equal_runs` in `functions/lift/{mod.rs,driver.rs}`. The kernel is `&mut dyn FnMut`, and the arguments stay a `Vec<CalcResult>`: the Range/Array → `LiftedArg` read is inside `lift_values` until T-1770's read phase and `ArgBuf`. `reuse_equal_runs` compares the grid slots at the previous position through `bcast_get` (no per-element key `Vec`), and one value buffer is kept for the call (scalar slots are written once). The cost stays today's: per-element cost × lifted arguments × computed elements. SPEC-0011's "elements × cells" for `Cells`/`Walk` would change which calls are refused, so T-1770 owns that change if it wants it. All callers (*IF(S), AGGREGATE, IFS, SWITCH, `lookup_each`, XLOOKUP) pass `LiftOptions::REUSE_EQUAL_RUNS`, as before. Unit tests cover `count_walks` with reuse on and off. No eval delta: `make eval-check` showed 0 fixed and 0 regressions in every baselined suite. The full `make eval` report.json on the parent and on the branch is identical, cell by cell, in all suites, including the suites with no baseline (gen-m6 to gen-m9, probes-numeric). `functions/mod.rs` stays at its locked size: one comment was folded into one line to make room for `mod lift;`.

Bench `crates/rustsheet/benches/lift.rs` (`cargo bench -p rustsheet --bench lift [-- NAME]`, release; column A holds runs of 10,000 equal values). It is a `harness = false` timer, not criterion (not a dependency): the timing and workbook set-up it shares with `aggregate.rs` and `text.rs` live in `benches/common/mod.rs`. Each process times one case: one warm-up and 11 recalcs, printing min, median and spread (max − min). Parent (origin/main 3d1aa007) and branch binaries were built from the same bench source and run interleaved in ABBA order, pinned to one CPU, 8 rounds per case (16 for SUMIFS), on a shared 4-core box with load 3–4 from another agent's build. Min is the lowest over all rounds, median is the median of the per-process medians, and spread is the median of the per-process spreads. Δ is the mean paired per-round change in min, with its 95% confidence interval:

| Case | rows | parent min / median / spread | branch min / median / spread | Δ min (95% CI) |
|---|---|---|---|---|
| `=SUM(COUNTIFS(A:A,A1:A100000))` (`reuse_equal_runs`) | 100,000 | 1030 / 1202 / 283 ms | 979 / 1159 / 335 ms | +0.1% ± 3.7% |
| `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))` | 100,000 | 973 / 1104 / 229 ms | 928 / 1079 / 237 ms | −3.3% ± 2.6% |
| `=SUM(SWITCH(A1:A1000000,0,1,1,2,3))` (driver-dominated) | 1,000,000 | 585 / 677 / 185 ms | 545 / 644 / 160 ms | −5.9% ± 3.3% |
| `=ROWS(SWITCH(A1:A1000000,0,"a","b"))` (text result clones) | 1,000,000 | 653 / 770 / 181 ms | 623 / 732 / 154 ms | −4.4% ± 3.8% |

No case regresses. The noise of the comparison (the 95% half-width) is under 5% in every case, and no interval reaches +5%: the highest upper bound is +3.8% (COUNTIFS). The two SWITCH cases, where the driver dominates, are about 5% faster, which fits dropping the per-element key `Vec`. A single process is noisy on this box: its spread is 15–30% and the per-process min varies by 5–15%. That is why the comparison uses paired, interleaved rounds rather than one run each. One artifact was seen: with all four cases in one process, the branch's SUMIFS (run after COUNTIFS) took about 600 ms against the parent's 1000 ms. Run on its own, as in the table, it is at parity. So that gap comes from process state (the allocator or the heap left by the previous case), not from the driver: SUMIFS lifts only 10 elements.
