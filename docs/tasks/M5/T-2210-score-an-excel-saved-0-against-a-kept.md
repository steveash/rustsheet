---
id: T-2210
title: "Score an Excel-saved 0 against a kept subnormal when a probe of the same formula has a FALSE =0 companion"
milestone: M5
lane: harness
status: done
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

## Outcome
- Decision 0090: a `cached_overrides` line may point a cached 0 at an excel-graph probe cell of the same formula on
  the same inputs; when that probe's own companion `=(<formula>)=0` is FALSE, the override carries decision 0042's
  kept-subnormal flag (`overrides::evidence`). The cross-case lookup reuses decision 0032's override file, so the
  harness gains no new format. `kept_nonzero_formulas` (case.rs) is the one definition of the companion rule,
  shared by spec cases and overrides; `CellInput::formula` replaces the runner's local closure.
- EXP_SIGN!B44 now scores against probes/subnormal-saved-as-zero S!A1 and passes. Its F line leaves
  eval/baseline/ironcalc-calc.txt and its exit-exception line goes (no other baseline line changes).
- Unit test `kept_nonzero_needs_a_false_graph_companion`: a TRUE companion, another formula's companion, an
  excel-desktop sidecar or a missing probe spec leaves the flag off, so a cached 0 against a subnormal still fails.
- Eval delta: +1 fixed (ironcalc-calc EXP_SIGN!B44), 0 regressions.
