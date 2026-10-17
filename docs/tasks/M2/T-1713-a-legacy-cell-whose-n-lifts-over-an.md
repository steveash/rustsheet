---
id: T-1713
title: "A legacy cell whose N lifts over an array is #VALUE!"
milestone: M2
lane: functions-logic
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-1727]
---

## Goal
A legacy (non-array) cell `=N(IF(1,{2;3}))` gives `#VALUE!`, as Excel does, instead of spilling 2, 3.

## Context
probes/n-array-lift S!D4:D5 (excel-graph, T-1616). T-1616 made N lift over an array (dynamic `=N(IF(1,{2;3}))` spills 2, 3, matching Excel), but a legacy cell holding the same formula is `#VALUE!` in Excel, not its top-left element, while legacy `SUM(N({1,2}))` is 3 and `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` 5. Likely shape: in the legacy rewrite (`add_implicit_intersection`, expressions/parser/static_analysis/legacy.rs) a top-level N over an array argument is `#VALUE!`; probe first whether other scalar-argument functions (ABS, LEN) behave the same in a legacy cell. docs/excel-semantics/information.md "N over an array lifts element-wise".

## Acceptance criteria
- [ ] probes/n-array-lift S!D4:D5, probes/legacy-array-result A10:A11, G10:G11, I10 and probes/legacy-n-t-reference-args
  A10, B10, E10, G10:I11, K10, L10, N10, O10, R10:T11 (and their FORMULATEXT cells in row 20 where they show no `@`) leave eval/baseline/probes.txt.
- [ ] information.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Progress (2026-10-16, in flight)
- Probe probes/legacy-array-result (excel-graph) answers "do other scalar-argument functions behave
  the same": no. ABS, LEN, ISNUMBER, negation and a top-level IF get an `@` in a legacy cell (top-left
  value), and so do N of an array constant and of TRANSPOSE. Only N and T over `IF(...)` get no `@`
  and are `#VALUE!` (A10, G10, I10). Recorded in references.md "Legacy formulas put `@` on
  array-valued calls and array constants". The general `@` is filed as T-1717 (engine, opus).
- Next: decide whether T-1713 lands before or with T-1717. A narrow fix needs the legacy evaluation
  of N/T to see an array argument that is not `@`-ed and give `#VALUE!`; check whether CHOOSE,
  INDEX or OFFSET arguments behave like IF (probe) before choosing the rule.
- 2026-10-16 (resumed): probe probes/legacy-n-t-reference-args (excel-graph) settles the rule: N and
  T over an array-valued IF, CHOOSE, INDEX or IFS get no `@` and are `#VALUE!`, also nested
  (`ABS(N(IF(...)))`, `N(IF(...))+0`) and for a one-element array; IFERROR gets an `@`; SWITCH gets
  none and spills; ranges are read as references (top-left). references.md "Legacy N and T over a
  reference-returning call".

## Blocked because
The legacy text is identical to the dynamic formula's (`=N(IF(1,{2;3}))`, no `@`), and the engine
keeps a legacy formula only as its dynamic twin plus the `@`s of `add_implicit_intersection`
(legacy_formula.rs, xlsx import), so evaluation cannot tell that this cell must be `#VALUE!` while
the dynamic one spills. That needs a persisted legacy marker, an architectural change that needs a
spec first (CLAUDE.md directive 4): T-1727. Resume this task once the spec's implementation lands.
The range half (`=N(Z1:Z2)` 7, no `@`) needs no marker and is T-1728.
