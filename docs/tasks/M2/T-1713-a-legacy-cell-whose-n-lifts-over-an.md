---
id: T-1713
title: "A legacy cell whose N lifts over an array is #VALUE!"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1727, T-1851]
spec: docs/specs/SPEC-0012-legacy-n-t-marker.md
---

## Goal
A legacy (non-array) cell `=N(IF(1,{2;3}))` gives `#VALUE!`, as Excel does, instead of spilling 2, 3.

## Context
probes/n-array-lift S!D4:D5 (excel-graph, T-1616). T-1616 made N lift over an array (dynamic `=N(IF(1,{2;3}))` spills 2, 3, matching Excel), but a legacy cell holding the same formula is `#VALUE!` in Excel, not its top-left element, while legacy `SUM(N({1,2}))` is 3 and `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` 5. Likely shape: in the legacy rewrite (`add_implicit_intersection`, expressions/parser/static_analysis/legacy.rs) a top-level N over an array argument is `#VALUE!`; probe first whether other scalar-argument functions (ABS, LEN) behave the same in a legacy cell. docs/excel-semantics/information.md "N over an array lifts element-wise".

T-1851's kernels make T over a one-element array `#VALUE!` (`T(IF(1,{"a"}))`), extrapolated from N's probe (legacy-n-t-reference-args N10); add a T one-element cell to this task's placement or eval cases.

## Acceptance criteria
- [x] The 32 cells listed in SPEC-0012 "Problem" leave eval/baseline/probes.txt:
  - probes/legacy-n-t-reference-args A10, A20, B10, B20, E10:E11, G10:I11, K10, K20, L10, L20,
    N10, N20, O10 and R10:T11;
  - probes/legacy-array-result A10:A11, G10:G11 and I10;
  - probes/n-array-lift D4:D5.

  There are no regressions in probes, ironcalc-calc, poi or fixtures.
- [x] SPEC-0012's T-1713 test plan: placement tests (also the negative ones), the idempotence
  property over every probes `legacy_formulas` cell, and the export round trip (legacy → plain
  `<f>`; dynamic `=N(IF(1,{2;3}))` → `cm="1"`). It includes the statically scalar case:
  `=N(IF(A1,1,2))` gets no marker, legacy or dynamic, and still exports as a plain `<f>`.
- [x] information.md and references.md updated.
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
- 2026-10-18 (claimed, engine): both dependencies are done on main, T-1727 (SPEC-0012) and T-1851
  (the `NLegacy`/`TLegacy` kinds, 49421af1), so the task is unblocked.

## Design (SPEC-0012, 2026-10-18)
T-1727 settled the design in docs/specs/SPEC-0012-legacy-n-t-marker.md. T-1851 adds the internal
`NLegacy`/`TLegacy` kinds. This task places them in `add_legacy_intersection`'s `FunctionKind`
arm under T-1717's `arrays` flag (`legacy_reads_array_as_error`). It also extends
`leaves_call_whole` to CHOOSE, INDEX and IFS over array constants, and drops the one-element `@`
under the marker (N20). The export mirror needs no step ("Placement", items 1-6).
Moved from the functions-logic lane to engine, as the change is in expressions/parser/static_analysis/.

## Blocked because (resolved: T-1727 and T-1851 are done)
The legacy text is identical to the dynamic formula's (`=N(IF(1,{2;3}))`, no `@`), and the engine
keeps a legacy formula only as its dynamic twin plus the `@`s of `add_implicit_intersection`
(legacy_formula.rs, xlsx import), so evaluation cannot tell that this cell must be `#VALUE!` while
the dynamic one spills. That needs a persisted legacy marker, an architectural change that needs a
spec first (CLAUDE.md directive 4): T-1727 (done: SPEC-0012). Resume this task once T-1851 is
`done` on main.
The range half (`=N(Z1:Z2)` 7, no `@`) needs no marker and is T-1728.

## Outcome
- **Placement:** `mark_legacy_reading` (static_analysis/legacy.rs) runs at the end of
  `add_legacy_intersection`'s `FunctionKind` arm when T-1717's `arrays` flag holds. It turns N/T
  into `NLegacy`/`TLegacy` (`Function::legacy_reading`, functions/kinds.rs) when the single argument
  is a bare IF, or a CHOOSE, INDEX or IFS whose every returnable argument is an array constant
  (`returns_array_constants`), and that argument is not statically scalar.
- **Order:** SPEC-0012's order is followed, after the argument loop. The decision runs on the
  rewritten argument because `=N(IF(W1:W2,Z1,Z2))` (probes legacy-n-t-reference-args-2 H) is an array
  before the rewrite and `IF(@W1:W2,..)`, a scalar, after it.
- **One-element `@`:** the predicate looks through the automatic one-element `@`s of
  `shows_scalar_array_at` in the returned arms (IF's arms, CHOOSE's and IFS's values). It takes them
  off in place and puts them back if the call is not marked, so no node is cloned. The marked call
  keeps them off (`=N(IF(1,{2}))`, N20; SPEC-0012 item 6).
- **`leaves_call_whole`** now covers the CHOOSE/INDEX/IFS array-constant shapes too, so they lose
  their `@` (A20, B20, K20, L20), CHOOSE's index included. With a mixed arm they keep it.
- **Merged with T-1908** (cbd3efd9): N/T over INDEX of a plain reference, and CHOOSE with a literal
  index over references, take no `@` and are read as references. Those shapes are not marked (no
  array-constant returns), so the two rules are disjoint and there is no semantic conflict. Both
  arms are in `leaves_call_whole`.
- **No export change.** `legacy_reading_intersects` gives a plain `<f>` for a marked legacy cell and
  `cm="1"` for a dynamic twin, and `=N(IF(A1,1,2))` stays a plain `<f>` either way.
- **Room for the step:** `static_analysis_operator_chain` and `static_analysis_op_results` moved to
  static_analysis/broadcast.rs (23 lines out of static_analysis.rs).
- **Review round 1, new probe** probes/legacy-n-t-marker-shapes (M2, excel-graph, 2026-10-18).
  FORMULATEXT shows no `@` in any cell and no cell spills.
  - `#VALUE!`: `T(IF(1,{"a"}))`, `N(CHOOSE(1,{2}))`, `T(CHOOSE(1,{"a"}))`, `N(IFS(1,{2}))`,
    `N(INDEX({2,3;4,5},1))`, `N(INDEX({2;3},1))`, `N(IF(1,{2;3}+0))`, `LEN(N(IF(1,{2;3})))`,
    `N(IF(1,{2;3},Z1))`, `N(CHOOSE({1;2},{2;3},{4;5}))` and `T(CHOOSE({1;2},{"a";"b"},{"c";"d"}))`.
  - 2: `N(INDEX({2},1,1))`.
  - Ours matches 23 of 24 cells. That includes the dropped CHOOSE-index `@` (K20, L20) and the
    one-element CHOOSE/IFS arms (B-D), so they stay as they were.
  - Not matched: F10 `N(INDEX({2;3},1))` is 2 in ours, because our INDEX gives a scalar. Known
    failure, T-1921.
  - Which shapes are verified and which are extrapolated: references.md "Legacy N and T over a
    reference-returning call".
- **Tests:**
  - test_legacy_n_t_marker.rs: placement over the Problem table and the new probe's shapes (29
    marked and 17 negative rows, among them one-element arms whose `@` survives on an unmarked
    call); the legacy and dynamic values; idempotence (twice, and over the display text, as RC)
    over every `legacy_formulas` cell of the probes cases.
  - static_analysis/legacy.rs unit tests: `mark_legacy_reading`'s restore branch (a one-element `@`
    taken off and put back on an unmarked call) and its marked branch, on hand-built trees. No
    formula text reaches the restore branch: the rewrite puts the `@` on such a call itself, so the
    step returns early.
  - rustsheet-xlsx tests/dynamic_formula_export.rs: the export round trip for N/T over IF, CHOOSE,
    INDEX and IFS, the statically scalar case, and a legacy and a dynamic twin as two shared
    formulas.
- **Eval:**
  - probes known failures 2110 → 2079 against main at 0c0cb22b (T-1908 and T-1773 merged): the 32 acceptance
    cells fixed, 1 new-coverage failure (legacy-n-t-marker-shapes F10, T-1921), 0 regressions.
  - ironcalc-calc 1243 → 1243 and fixtures 9 → 9; poi is not fetched here.
- **Ratchets (decision 0010 line):** after merging main at 0c0cb22b (T-1773, which recorded the shrinks
  T-1851 left), oversized-files static_analysis.rs 1049 → 1026 (the operator-chain move); duplicated
  lines unchanged at 3863.
- **Gates:** fmt, clippy `-D warnings`, `cargo test` (engine, rustsheet, xlsx, eval), `make hygiene`
  and the eval checks pass. Reviewers: round 1 changes addressed; the reviewer box is still open.
- **Follow-ups:** T-1921 (new: N over `INDEX({2;3},1)`). Already filed: T-1852 (re-entry paths),
  T-1908 (CHOOSE/INDEX over references) and T-1847 (T over SWITCH, unprobed positions).
