---
id: T-1876
title: "Settle the target for desktop SIN(4) in ironcalc-calc IF_ARRAY (one ulp from web)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The three known failures of `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx` (`Sheet1!C4`, `E4`, `C11`) get
an owner and a decided target. All three are desktop SIN(4) = -0.7568024953079283. Excel for the
web and our engine give -0.7568024953079282, the correctly rounded value. C11 counts toward M2's
exit: it is a member of `IF(FALSE,A1:A4,B1:C4)`, which calls only IF (see T-1877). No open task
owns any of the three cells today.

## Context
- Evidence: desktop cached values (Microsoft Macintosh Excel 16.0300) in
  crates/rustsheet-xlsx/tests/calc_tests/LOGICAL/IF_ARRAY.xlsx. The web value is in
  docs/research/oracle-calibration.md ("spilled SIN(4)", 3 array cells). docs/excel-semantics/numbers.md
  has "Excel for the web and desktop Excel differ in the last digits", which lists SIN.
- Scalar `=SIN(4)` and `=SIN(A1)` with A1 = 4 also give ...282 in our engine, so the gap is the
  SIN kernel, not the array path. Checked 2026-10-18 with a temporary hand case; the lifted and
  INDEX forms agree.
- The SIN kernel (`functions/math_and_trigonometry/trig.rs`, T-1563) is bit-exact on 672 web
  cells (probes-numeric/m9-trig-reduction, probes/trig-huge-argument). It also passes the
  desktop SIN cells of ironcalc-calc TRIGONOMETRIC.xlsx `SIN_COS_TAN` (about 50 cells, also Mac
  16.x).
- Decision 0030 rule 4: an engine fit targets desktop cached values where they exist. Decision
  0039 amends it: in Phases 1–3, a conflict that needs new desktop evidence matches the web value.
- Tools: `rustsheet-fitlab` (docs/architecture/fit-lab.md; `fitlab/exact.py` has the x87 model).

## Acceptance criteria
- [x] Fit-lab run, recorded in numbers.md: does any candidate SIN formulation give the desktop
  SIN(4) and keep every cell the T-1563 kernel passes today? Candidates include x87 `fsin` over
  the T-1563 reduction, the reduction with a 64-bit intermediate, and Apple libm-style
  reductions. Report the counts per candidate.
- [x] (n/a) If such a formulation exists: a functions-math task (opus) is filed to implement it, naming
  the three IF_ARRAY cells as its acceptance cells.
- [x] Otherwise: a decision record (`make new-decision`) applies decisions 0030/0039 to this
  cell. Either the engine keeps the web value, and the three cells are named as Excel-vs-Excel
  last-digit conflicts in numbers.md, or another resolution the record justifies. If the record
  changes how M2's exit counts them, docs/milestones.md says so. Directive 2 still holds: no
  `--allow-regressions`, no baseline edit.
- [x] numbers.md entry updated with the SIN(4) evidence and the outcome. `make check` green;
  reviewers approve.

## Out of scope
- Implementing a new SIN kernel (the follow-up task).
- Other functions in the desktop/web last-digit list (ATAN2, IM*, ERFC, BESSEL, MIRR, IRR).

## Outcome
No SIN formulation gives desktop SIN(4) = -0.7568024953079283 while keeping the cells the T-1563 kernel passes.
- The fit used exact-arithmetic models plus glibc and an fdlibm/musl port, scored on 224 web SIN/COS cells and on the
  38 desktop cells of TRIGONOMETRIC.xlsx. Per-candidate counts are in numbers.md "Excel for the web and desktop Excel
  differ in the last digits".
  - The engine's model scores 224/224 and 38/38.
  - Every candidate gives ...282, except a double-π reduction (...284).
  - The true sin(4) is 0.06 ulp from the midpoint. The 38 desktop cells are all correctly rounded, so the Mac kernel is
    only faithful, and one cell cannot pin it down.
- Decision 0074: the engine keeps the web value. C4, E4 and C11 stay known failures, named as an Excel-vs-Excel conflict.
  Their desktop question is item 11 of T-1075 (M14), which counts as a filed task for M2's ironcalc-calc criterion
  (note added in docs/milestones.md).
- Eval delta: none (docs only).
