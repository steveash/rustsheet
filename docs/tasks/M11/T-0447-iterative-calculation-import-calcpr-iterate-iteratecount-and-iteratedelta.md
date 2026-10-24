---
id: T-0447
title: "Iterative calculation: import calcPr iterate, iterateCount and iterateDelta and sweep cycles in place"
milestone: M11
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0446]
---

## Goal
A workbook saved with iterative calculation on (`<calcPr iterate="1" iterateCount=… iterateDelta=…>`)
calculates its circular references as Excel does: the cycle's cells are swept in row-major sheet order,
in place, up to iterateCount times, stopping at the first sweep whose largest change is below
iterateDelta. They are no longer `#CIRC!`.

## Context
Found by the mog head-to-head (docs/research/mog-head-to-head-2026-10-22.md):
- Two real financial models in the Calipers corpus, pv23_021 and pv26_004 (T-0446), are saved with
  `iterate="1"`. Rustsheet makes about 3,600 of their formula cells `#CIRC!`, its largest
  wrong-type group on that corpus. Mog and Excel compute numbers.
- The importer ignores `iterate`, `iterateCount` and `iterateDelta`, and the engine has no iterative
  mode, so every cycle cell is an error.

The rules are already established, with probes:
- docs/excel-semantics/recalc.md, "Iterative calculation sweeps the cycle in sheet order, in place" and
  "iterateDelta stops the iterations for the whole workbook, not per cycle" (both VERIFIED, excel-graph).
- probes/circular-iteration-on, -limits, -delta, -delta-small, -delta-no-reader, -delta-boundary.
- The harness already runs spec cases with `calculation.iterate` three times, the oracle's count
  (T-0429, `calculations` in crates/rustsheet-eval/src/engine.rs).
- spec2xlsx writes the `calcPr` attributes, so the probes and the Calipers models exercise the same
  import path.

Code: the cycle detection and the `#CIRC!` paths are in crates/rustsheet-engine/src/model.rs
(`run_pass`); calcPr import is in crates/rustsheet-xlsx/src/import.

Coordinate with:
- T-0445 (iteration off: cycle cells and their readers are 0). It changes the same detection path;
  whichever lands second rebases onto the other.
- T-0401 (SPEC-0002). Do not build a dependency graph here. Sweeping the cells found on a cycle during
  a pass is enough for Excel's order.

Directive 5: the sweep count is bounded by iterateCount (Excel caps it at 32767), and each sweep by the
existing per-pass work budget.

## Acceptance criteria
- [x] The probes/circular-iteration-on, -limits, -delta, -delta-small, -delta-no-reader and
      -delta-boundary lines leave eval/baseline/probes.txt (`make eval-baseline`).
- [x] calipers/pv_goldens/pv23_021 and pv26_004 have no `#CIRC!` cells. Their known failures in
      eval/baseline/calipers.txt drop (`rustsheet-eval update-baseline --suite calipers`) accordingly, with no new known failure in any suite.
- [x] A round-trip test keeps `iterate`, `iterateCount` and `iterateDelta` on export.
- [x] recalc.md's two "Ours" lines state the new behavior.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cycles with iteration off (T-0445).
- Data tables inside a cycle (T-0441).

## Outcome
- Settings: `WorkbookSettings::iteration` (`Iteration { max_iterations, max_change }`). The xlsx
  import reads `calcPr@iterate`, `iterateCount` and `iterateDelta`, out-of-range values falling back
  to Excel's defaults; the export writes them back (round-trip test). The facade has
  `Workbook::set_iteration`, and the eval adapter sets it from a case's `calculation`.
- Engine (crates/rustsheet-engine/src/iteration.rs):
  - With iteration on, a read that closes a cycle returns the cell's current value instead of
    `#CIRC!`, whatever evaluates it. On-demand evaluations count too, such as conditional
    formatting at load, which had turned the cycles into stored `#CIRC!`.
  - The stack at that moment joins the recalculated set, and so does every later reader of a
    member.
  - After the pass the members get back their previous values (0 if never evaluated). They are then
    swept in row-major order, Gauss-Seidel, until the largest change is strictly below the delta or
    iterateCount is reached.
  - Directive 5: at most 2^25 member evaluations per recalculation (recalc.md DEVIATION).
- `Restart`/`RestartLog` moved to evaluation/restart.rs to make room; evaluation.rs is down to
  about 880 lines.
- Eval: the six probes/circular-iteration-on/-limits/-delta/-delta-small/-delta-no-reader/
  -delta-boundary pass. Calipers models with iteration on match Excel: pv23_021, pv26_004, pv23_019,
  pv23_027 and pv26_012 at 100%, pv26_014 at 98.6%. The rest of pv26_014 is IRR precision (T-2131)
  and four data-table results in an iterating model (T-0454).

