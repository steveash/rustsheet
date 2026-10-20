---
id: T-2210
title: "Score an Excel-saved 0 against a kept subnormal when a probe of the same formula has a FALSE =0 companion"
milestone: M5
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The harness scores an Excel-saved 0 (excel-cached or excel-graph) against a kept subnormal engine value as a pass when
Excel's own evidence elsewhere shows the same formula over the same inputs keeps a non-zero value, so ironcalc-calc
`MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx` EXP_SIGN!B44 passes and its decision 0089 exit-exception line can go.

## Context
- Decision 0089: Excel writes a kept subnormal to the saved xlsx as `<v>0</v>` (probes/subnormal-saved-as-zero). B44 is
  `=EXP(A44)`, A44 = -745 (stored), cached 0 by Excel Online; the engine gives 2^-1074, which is Excel's in-memory
  value (probes/subnormal-saved-as-zero B1 FALSE, C1 = 2^-74).
- Decision 0042's rule (`crates/rustsheet-eval/src/runner.rs` `graph_hides_subnormal`) needs a FALSE `(x)=0`
  companion in the same case and an excel-graph sidecar, so it never reaches an excel-cached xlsx cell.
- The rule this task needs is cross-case: a cell's formula evaluated over its inputs must match a probe cell whose
  `(x)=0` companion is FALSE. One design: a small checked-in evidence list (formula text with inputs folded in, e.g.
  `EXP(-745)`) built from probe companions, with B44 resolved through its precedent A44. Weigh that against how
  few cells need it; if the design is not worth its code, say so in a decision and leave the exception in place.
- The rule must not pass a flushed result: where the companion is TRUE (`*`, `/`, PRODUCT, ...) the cell still fails.
- eval/milestones/ and crates/rustsheet-eval scoring are protected or gate-adjacent: the change needs a decision
  record extending decision 0042.

## Acceptance criteria
- [ ] ironcalc-calc `MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx` EXP_SIGN!B44 passes and leaves eval/baseline/ironcalc-calc.txt
      (`make eval-baseline`); its line leaves eval/milestones/exit-exceptions.tsv.
- [ ] A unit test shows a cached 0 against a subnormal with no FALSE-companion evidence still fails.
- [ ] A decision record extends decision 0042; no other baseline line changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes to EXP (T-1944, T-2203).
